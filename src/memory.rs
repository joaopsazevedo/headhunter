pub fn get_process_ids(process_name: &str) -> Vec<u32> {
    // TODO: handle process_name with and without .exe suffix

    let handle: windows_sys::Win32::Foundation::HANDLE = unsafe {
        windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS,
            0,
        )
    };

    if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        // TODO: Return error
    }

    let mut process_ids = vec![];

    let mut current_process = windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32 {
        dwSize: std::mem::size_of::<windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32>() as u32,
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
        let current_process_name = std::str::from_utf8(&current_process.szExeFile).unwrap();
        //  TODO: Log if we could not convert the process .exe name to UTF16
        if current_process_name == process_name {
            process_ids.push(current_process.th32ProcessID)
        }
    }

    unsafe {
        if windows_sys::Win32::Foundation::CloseHandle(handle) == 0 {
            // TODO: Log that we couldn't close the snapshot handle.
        }
    }

    process_ids
}
