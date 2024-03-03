use crate::memory;
use anyhow::anyhow;

#[derive(thiserror::Error, Debug)]
pub enum MemoryError {
    #[error("failed to get processes snapshot")]
    ProcessSnapshotFailed,
    #[error("CS2 process not found")]
    CS2ProcessNotFound,
    #[error("multiple CS2 processes found (only one allowed)")]
    MultipleCS2Processes,
}

pub fn get_process_ids(process_name: &str) -> Result<Vec<u32>, MemoryError> {
    let handle: windows_sys::Win32::Foundation::HANDLE = unsafe {
        windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS,
            0,
        )
    };

    if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(MemoryError::ProcessSnapshotFailed);
    }

    let mut process_ids = vec![];

    let mut current_process = windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32 {
        dwSize: std::mem::size_of::<windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32>(
        ) as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; 260],
    };
    while unsafe {
        windows_sys::Win32::System::Diagnostics::ToolHelp::Process32Next(
            handle,
            &mut current_process,
        ) != 0
    } {
        match std::ffi::CStr::from_bytes_until_nul(&current_process.szExeFile) {
            Ok(current_process_name_cstr) => match current_process_name_cstr.to_str() {
                Ok(current_process_name) => {
                    log::trace!("Found process {current_process_name}");
                    if current_process_name == process_name {
                        process_ids.push(current_process.th32ProcessID)
                    }
                }
                Err(_) => {
                    log::warn!(
                        "failed to convert process {} name from CStr to &str",
                        current_process.th32ProcessID
                    )
                }
            },
            Err(_) => {
                log::warn!(
                    "failed to get process {} name",
                    current_process.th32ProcessID
                )
            }
        }
    }

    unsafe {
        if windows_sys::Win32::Foundation::CloseHandle(handle) == 0 {
            log::warn!("failed to close snapshot handle")
        }
    }

    Ok(process_ids)
}

pub fn get_cs2_process_id() -> Result<u32, MemoryError> {
    let mut process_ids = memory::get_process_ids("cs2.exe")?.into_iter();

    if let Some(process_id) = process_ids.next() {
        if process_ids.next().is_some() {
            return Err(MemoryError::MultipleCS2Processes);
        } else {
            Ok(process_id)
        }
    } else {
        Err(MemoryError::CS2ProcessNotFound)
    }
}
