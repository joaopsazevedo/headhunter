use crate::memory;

#[derive(thiserror::Error, Debug)]
pub enum MemoryError {
    #[error("failed to get processes snapshot")]
    ProcessSnapshotFailed,
    #[error("CS2 process not found")]
    CS2ProcessNotFound,
    #[error("multiple CS2 processes found (only one allowed)")]
    MultipleCS2Processes,
    #[error("failed to find module {0}")]
    FailedToFindModule(String),
}

unsafe fn get_process_ids(process_name: &str) -> Result<Vec<u32>, MemoryError> {
    let handle: windows_sys::Win32::Foundation::HANDLE =
        windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS,
            0,
        );

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
    while {
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
                        "failed to convert process {} name from &CStr to &str",
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

    if windows_sys::Win32::Foundation::CloseHandle(handle) == 0 {
        log::warn!("failed to close process snapshot handle")
    }

    Ok(process_ids)
}

unsafe fn get_module_base_address(pid: u32, module_name: &str) -> Result<usize, MemoryError> {
    let handle: windows_sys::Win32::Foundation::HANDLE =
        windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPMODULE,
            pid,
        );

    if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(MemoryError::ProcessSnapshotFailed);
    }

    let mut module_entry = windows_sys::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32 {
        dwSize: std::mem::size_of::<windows_sys::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32>(
        ) as u32,
        th32ModuleID: 0,
        th32ProcessID: 0,
        GlblcntUsage: 0,
        ProccntUsage: 0,
        modBaseAddr: std::ptr::null_mut(),
        modBaseSize: 0,
        hModule: 0,
        szModule: [0; windows_sys::Win32::System::Diagnostics::ToolHelp::MAX_MODULE_NAME32
            as usize
            + 1],
        szExePath: [0; windows_sys::Win32::Foundation::MAX_PATH as usize],
    };

    while windows_sys::Win32::System::Diagnostics::ToolHelp::Module32Next(handle, &mut module_entry)
        != 0
    {
        match std::ffi::CStr::from_bytes_until_nul(&module_entry.szModule) {
            Ok(current_module_cstr) => match current_module_cstr.to_str() {
                Ok(current_module) => {
                    log::trace!("Found module {current_module}");
                    if current_module == module_name {
                        if windows_sys::Win32::Foundation::CloseHandle(handle) == 0 {
                            log::warn!("failed to close process module snapshot handle");
                        }
                        return Ok(module_entry.modBaseAddr as usize);
                    }
                }
                Err(_) => {
                    log::warn!("failed to convert module name from &CStr to &str");
                }
            },
            Err(_) => {
                log::warn!("failed to get module name");
            }
        };
    }

    if windows_sys::Win32::Foundation::CloseHandle(handle) == 0 {
        log::warn!("failed to close process module snapshot handle");
    }

    Err(MemoryError::FailedToFindModule(module_name.to_string()))
}

pub struct CS2Process {
    pub pid: u32,
    pub base_address: usize,
}

pub unsafe fn get_cs2_process_id() -> Result<CS2Process, MemoryError> {
    let pid = {
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
    }?;
    let base_address = get_module_base_address(pid, "client.dll")?;
    Ok(CS2Process { pid, base_address })
}
