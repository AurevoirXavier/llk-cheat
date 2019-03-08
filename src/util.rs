// --- external ---
use winapi::{
    shared::{
        minwindef::{LPCVOID, LPVOID},
        windef::HWND,
    },
    um::winnt::HANDLE,
};

const WINDOW_NAME: &'static str = "Untitled - Notepad";

#[derive(Debug)]
pub enum CheatError {
    FindWindowError,
    OpenProcessError,
    ReadProcessMemoryError,
    WriteProcessMemoryError,
    VirtualAllocError,
    CreateRemoteThreadError,
//    MessageBoxError,
}

pub fn find_window() -> Result<HWND, CheatError> {
    // --- external ---
    use winapi::{
        shared::minwindef::LPARAM,
        um::winuser::{EnumWindows, FindWindowW},
    };

    extern "system" fn enum_windows_proc(hwnd: HWND, l_param: LPARAM) -> i32 {
        // --- std ---
        use std::{
            ffi::OsString,
            os::windows::ffi::OsStringExt,
        };
        // --- external ---
        use winapi::um::winuser::GetWindowTextW;

        let mut buffer = [0; 128];
        let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as _) };
        let name = OsString::from_wide(&buffer[..len as _]);

        if name.to_str().unwrap() == WINDOW_NAME {
            unsafe { *(l_param as *mut HWND) = hwnd; }
            0
        } else { 1 }
    }

    let mut hwnd = unsafe {
        FindWindowW(
            0 as _,
            WINDOW_NAME.encode_utf16()
                .collect::<Vec<u16>>()
                .as_ptr(),
        )
    };
    if hwnd.is_null() {
        unsafe { EnumWindows(Some(enum_windows_proc), &mut hwnd as *mut HWND as _); }
        if hwnd.is_null() { return Err(CheatError::FindWindowError); }
    }

    Ok(hwnd)
}

pub fn find_process_id(hwnd: HWND) -> u32 {
    // --- external ---
    use winapi::{
        shared::minwindef::LPDWORD,
        um::winuser::GetWindowThreadProcessId,
    };

    let ptr: LPDWORD = &mut 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, ptr);
        *ptr
    }
}

pub fn open_process(process_id: u32) -> Result<HANDLE, CheatError> {
    // --- external ---
    use winapi::{
        shared::minwindef::TRUE,
        um::{
            processthreadsapi::OpenProcess,
            winnt::PROCESS_ALL_ACCESS,
        },
    };

    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, TRUE, process_id) };
    if handle.is_null() { Err(CheatError::OpenProcessError) } else { Ok(handle) }
}

pub fn read_process_memory(handle: HANDLE, address: LPVOID, buffer: LPVOID, size: usize) -> Result<(), CheatError> {
    // --- external ---
    use winapi::um::memoryapi::ReadProcessMemory;

    let result = unsafe { ReadProcessMemory(handle, address, buffer, size, 0 as _) };
    if result == 0 { Err(CheatError::ReadProcessMemoryError) } else { Ok(()) }
}

pub fn write_process_memory<T>(handle: HANDLE, address: LPVOID, buffer: *const T, size: usize) -> Result<(), CheatError> {
    // --- external ---
    use winapi::um::memoryapi::WriteProcessMemory;

    let result = unsafe { WriteProcessMemory(handle, address, buffer as LPCVOID, size, 0 as _) };
    if result == 0 { Err(CheatError::WriteProcessMemoryError) } else { Ok(()) }
}
