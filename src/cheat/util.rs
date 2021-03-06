// --- std ---
use std::collections::HashMap;
// --- external ---
use winapi::{
    shared::{
        minwindef::{FARPROC, LPCVOID, LPVOID},
        windef::HWND,
    },
    um::winnt::HANDLE,
};

const WINDOW_NAME: &'static str = "连连看5";

#[derive(Debug)]
pub enum CheatError {
    FindWindowError,
    GetWindowThreadProcessIdError,
    OpenProcessError,
    ReadProcessMemoryError,
    WriteProcessMemoryError,
    VirtualAllocError,
    CreateRemoteThreadError,
    GetModuleHandleError,
    GetProcAddressError,
    Exit,
}

pub struct Processes(pub HashMap<String, FARPROC>);

impl Processes {
    pub fn new() -> Processes { Processes(HashMap::new()) }

    fn get_proc_address(module: &str, process: &str) -> Result<FARPROC, CheatError> {
        // --- external ---
        use winapi::um::libloaderapi::{GetProcAddress, GetModuleHandleA};

        let h_instance = unsafe { GetModuleHandleA(&*module.to_owned() as *const _ as _) };
        if h_instance.is_null() { return Err(CheatError::GetModuleHandleError); }

        let ptr = unsafe { GetProcAddress(h_instance, &*process.to_owned() as *const _ as _) };
        if ptr.is_null() { Err(CheatError::GetProcAddressError) } else { Ok(ptr) }
    }

    pub fn add(mut self, module: &str, process: &str) -> Self {
        let process_ptr;
        loop {
            match Processes::get_proc_address(module, process) {
                Ok(ptr) => {
                    process_ptr = ptr;
                    break;
                }
                Err(e) => println!("{:?}", e)
            }
        }
        self.0.insert(process.to_owned(), process_ptr);

        self
    }
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

pub fn get_window_thread_process_id(hwnd: HWND) -> Result<u32, CheatError> {
    // --- external ---
    use winapi::{
        shared::minwindef::LPDWORD,
        um::winuser::GetWindowThreadProcessId,
    };

    let ptr: LPDWORD = &mut 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, ptr);
        if ptr == 0 as _ { Err(CheatError::GetWindowThreadProcessIdError) } else { Ok(*ptr) }
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

pub fn create_remote_thread(handle: HANDLE, proc: LPVOID, param: LPVOID) -> Result<(), CheatError> {
    // --- external ---
    use winapi::um::processthreadsapi::CreateRemoteThread;
    // --- custom ---
    use super::remote::RemoteProc;

    let handle = unsafe { CreateRemoteThread(handle, 0 as _, 0, Some(*(&proc as *const _ as *const RemoteProc)), param, 0, 0 as _) };
    if handle.is_null() { Err(CheatError::CreateRemoteThreadError) } else { Ok(()) }
}
