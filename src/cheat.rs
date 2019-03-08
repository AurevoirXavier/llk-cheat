// --- external ---
use winapi::{
    shared::minwindef::{LPDWORD, LPVOID},
    um::winnt::HANDLE,
};
// --- custom ---
use super::util::*;
use winapi::um::errhandlingapi::{GetLastError, SetLastError};

const BASE_ADDRESS: u32 = 0x4C0E2C;

fn inject() -> Result<HANDLE, CheatError> {
    let hwnd = find_window()?;
    println!("hwnd: {:?}", hwnd);

    let process_id = find_process_id(hwnd);
    println!("process id: {}", process_id);

    let handle = open_process(process_id)?;
    println!("handle: {:?}", handle);

    Ok(handle)
}

fn get_ptr(handle: HANDLE, offsets: Vec<u32>) -> Result<LPVOID, CheatError> {
    let mut ptr = BASE_ADDRESS + offsets[0];
    for offset in offsets[1..].iter() {
        let buffer: LPDWORD = &mut 0;
        read_process_memory(handle, ptr as _, buffer as _, 4)?;
        unsafe { ptr = *buffer + offset; }
    }

    Ok(ptr as _)
}

fn hack_game_timer(handle: HANDLE, value: u32) -> Result<u32, CheatError> {
    //  exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x2C]
    let ptr = get_ptr(handle, vec![0x14, 0x20, 0x2C])?;
    write_process_memory(handle, ptr, &value as *const u32, 4)?;

    let time: LPDWORD = &mut 0;
    read_process_memory(handle, ptr, time as _, 4)?;

    unsafe { Ok(*time) }
}

fn hack_chance(handle: HANDLE, value: u32) -> Result<u32, CheatError> {
    // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x3C]
    let ptr = get_ptr(handle, vec![0x14, 0x3FB8, 0x3C])?;
    write_process_memory(handle, ptr, &value as *const u32, 4)?;

    let chance: LPDWORD = &mut 0;
    read_process_memory(handle, ptr, chance as _, 4)?;

    unsafe { Ok(*chance) }
}

fn hack_tip(handle: HANDLE, value: u32) -> Result<u32, CheatError> {
    // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x38]
    let ptr = get_ptr(handle, vec![0x14, 0x3FB8, 0x38])?;
    write_process_memory(handle, ptr, &value as *const u32, 4)?;

    let tip: LPDWORD = &mut 0;
    read_process_memory(handle, ptr, tip as _, 4)?;

    unsafe { Ok(*tip) }
}

fn hack_score(handle: HANDLE, value: u32) -> Result<u32, CheatError> {
    // exp = [[0x004C0E2C + 0x14] + 0x3F34]
    let ptr = get_ptr(handle, vec![0x14, 0x3F34])?;
    write_process_memory(handle, ptr, &value as *const u32, 4)?;

    let score: LPDWORD = &mut 0;
    read_process_memory(handle, ptr, score as _, 4)?;

    unsafe { Ok(*score) }
}

fn hack_combo_timer(handle: HANDLE, value: f32) -> Result<f32, CheatError> {
    // exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x30]
    let ptr = get_ptr(handle, vec![0x14, 0x20, 0x30])?;
    write_process_memory(handle, ptr, &value as *const f32, 4)?;

    let time: LPDWORD = &mut 0;
    read_process_memory(handle, ptr, time as _, 4)?;

    unsafe { Ok(*(time as *const f32)) }
}

fn hack_cells(handle: HANDLE) -> Result<[u32; 400], CheatError> {
    // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x180]
    let ptr = get_ptr(handle, vec![0x14, 0x3FB8, 0x180])?;
    let mut cells = [0; 400];

    read_process_memory(handle, ptr, cells.as_mut_ptr() as _, 1600)?;

    Ok(cells)
}

fn send_spy(handle: HANDLE) -> Result<(), CheatError> {
    // --- std ---
    use std::mem::size_of;
    // --- external ---
    use winapi::{
        shared::{
            minwindef::FARPROC,
            windef::HWND,
        },
        um::{
            libloaderapi::{LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, GetProcAddress, GetModuleHandleA, LoadLibraryExA},
            processthreadsapi::CreateRemoteThread,
            memoryapi::VirtualAllocEx,
            winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE, LPCSTR},
            winuser::{MB_OK, MessageBoxA},
        },
    };

    type MessageBoxProc = unsafe extern "system" fn(HWND, LPCSTR, LPCSTR, u32) -> i32;
    struct Param<'a> {
        cmd: u8,
        message_box_proc: FARPROC,
        message: &'a str,
    }

    let param_ptr = {
        let size = size_of::<Param>();
        unsafe {
            println!("{}", LoadLibraryExA("user32".as_ptr() as _, 0 as _, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS).is_null());
        }
        let param = Param {
            cmd: 0,
            message_box_proc: unsafe { GetProcAddress(LoadLibraryExA("user32".as_ptr() as _, 0 as _, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS), "MessageBoxA" as *const str as _) },
            message: "Hello game",
        };
        let ptr = unsafe { VirtualAllocEx(handle, 0 as _, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };

        if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else {
            write_process_memory(handle, ptr, &param as *const Param, size)?;
            ptr
        }
    };

    type RemoteThreadProc = unsafe extern "system" fn(LPVOID) -> u32;
    extern "system" fn remote_thread_proc(l_param: LPVOID) -> u32 {
        let param = unsafe { &*(l_param as *const Param) };
        if param.cmd == 0 {
            let message = param.message as *const str as *const i8;
//            unsafe { (*(param.message_box_proc as *mut MessageBoxProc))(0 as _, message, message, MB_OK); }
        }

        0
    }

    let proc_ptr = {
        let size = 0x1000;
        let ptr = unsafe { VirtualAllocEx(handle, 0 as _, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };

        if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else {
            write_process_memory(handle, ptr, remote_thread_proc as *const RemoteThreadProc, size)?;
            ptr
        }
    };

    let handle = unsafe { CreateRemoteThread(handle, 0 as _, 0, Some(*(&proc_ptr as *const _ as *const RemoteThreadProc)), param_ptr, 0, 0 as _) };
    if handle.is_null() { Err(CheatError::CreateRemoteThreadError) } else { Ok(()) }
}

pub fn option() -> Result<(), CheatError> {
    // --- std ---
    use std::io::{Write, stdin, stdout};

    print!("1.hack game time\n2.hack chance\n3.hack tip\n4.hack score\n5.hack combo time\n6.hack cells\n7.send spy\nfunction: ");
    stdout().flush().unwrap();
    let mut function = String::new();
    stdin().read_line(&mut function).unwrap();

    let handle = inject()?;

    match function.trim() {
        "1" => {
            let time = hack_game_timer(handle, 200)?;
            println!("time: {}", time);
        }
        "2" => {
            let chance = hack_chance(handle, 200)?;
            println!("chance: {}", chance);
        }
        "3" => {
            let tip = hack_tip(handle, 200)?;
            println!("tip: {}", tip);
        }
        "4" => {
            let score = hack_score(handle, 2000)?;
            println!("score: {}", score);
        }
        "5" => {
            let combo_time = hack_combo_timer(handle, 1000.)?;
            println!("combo_time: {}", combo_time);
        }
        "6" => {
            let cells = hack_cells(handle)?;
            println!("cells: [");
            for chunk in cells.chunks(50) {
                print!("    ");
                for cell in chunk { if *cell != 0xffffffff { print!("{:2}, ", cell); } }
                println!();
            }
            print!("]");
        }
        "7" => { send_spy(handle)?; }
        _ => ()
    }

    Ok(())
}
