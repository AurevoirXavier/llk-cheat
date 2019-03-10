// --- external ---
use winapi::{
    shared::{
        windef::HWND,
        minwindef::{FARPROC, LPDWORD, LPVOID},
    },
    um::winnt::{HANDLE, LPCWSTR},
};
// --- custom ---
use super::util::*;

const BASE_ADDRESS: u32 = 0x4C0E2C;

type MessageBoxProc = unsafe extern "system" fn(HWND, LPCWSTR, LPCWSTR, u32) -> i32;

struct RemoteParam {
    message_box_proc: FARPROC,
    message_box_title: [u16; 50],
    message_box_content: [u16; 50],
}

impl RemoteParam {
    fn new() -> RemoteParam {
        RemoteParam {
            message_box_proc: 0 as _,
            message_box_title: [0; 50],
            message_box_content: [0; 50],
        }
    }

    fn message_box_proc(mut self, ptr: FARPROC) -> Self {
        self.message_box_proc = ptr;
        self
    }

    fn message_box_title(mut self, s: &str) -> Self {
        let mut slice = s.encode_utf16().collect::<Vec<u16>>();
        slice.resize(self.message_box_title.len(), 0);
        self.message_box_title.clone_from_slice(&slice);

        self
    }

    fn message_box_content(mut self, s: &str) -> Self {
        let mut slice = s.encode_utf16().collect::<Vec<u16>>();
        slice.resize(self.message_box_content.len(), 0);
        self.message_box_content.clone_from_slice(&slice);

        self
    }
}

type RemoteThreadProc = unsafe extern "system" fn(LPVOID) -> u32;

extern "system" fn remote_thread_proc(l_param: LPVOID) -> u32 {
    let param = unsafe { &*(l_param as *const RemoteParam) };
    unsafe { (*(&param.message_box_proc as *const _ as *const MessageBoxProc))(0 as _, &param.message_box_content as *const [u16] as *const u16, &param.message_box_title as *const [u16] as *const u16, 0); }

    0
}

struct Cheat {
    process: Processes,
    target_proc: HANDLE,
    remote_ptrs: (LPVOID, LPVOID),
}

impl Cheat {
    fn new() -> Cheat {
        Cheat {
            process: Processes::new().add("user32.dll", "MessageBoxW"),
            target_proc: 0 as _,
            remote_ptrs: (0 as _, 0 as _),
        }
    }

    fn inject(&mut self) -> Result<(), CheatError> {
        // --- std ---
        use std::mem::size_of;
        // --- external ---
        use winapi::um::{
            processthreadsapi::CreateRemoteThread,
            memoryapi::VirtualAllocEx,
            winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE},
        };

        {
            let hwnd = find_window()?;
            let process_id = get_window_thread_process_id(hwnd)?;

            self.target_proc = open_process(process_id)?;
        }
        {
            let size = size_of::<RemoteParam>();
            let ptr = unsafe { VirtualAllocEx(self.target_proc, 0 as _, size, MEM_COMMIT, PAGE_READWRITE) };
            let param = RemoteParam::new()
                .message_box_proc(*self.process.0.get("MessageBoxW").unwrap())
                .message_box_title("Author: Xavier")
                .message_box_content("Inject succeed");
            if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else { write_process_memory(self.target_proc, ptr, &param as *const RemoteParam, size)?; }

            self.remote_ptrs.0 = ptr;
        }
        {
            let size = 0x1000;
            let ptr = unsafe { VirtualAllocEx(self.target_proc, 0 as _, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };
            if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else { write_process_memory(self.target_proc, ptr, remote_thread_proc as *const RemoteThreadProc, size)?; }

            self.remote_ptrs.1 = ptr;
        }
        let handle = unsafe { CreateRemoteThread(self.target_proc, 0 as _, 0, Some(*(&self.remote_ptrs.1 as *const _ as *const RemoteThreadProc)), self.remote_ptrs.0, 0, 0 as _) };

        if handle.is_null() { Err(CheatError::CreateRemoteThreadError) } else { Ok(()) }
    }

    fn exit(&mut self) {
        // --- external ---
        use winapi::um::handleapi::CloseHandle;

        self.target_proc = 0 as _;
        self.remote_ptrs = (0 as _, 0 as _);
        unsafe { CloseHandle(self.target_proc); }
    }

    fn read_process_memory(&self, address: LPVOID, buffer: LPVOID, size: usize) -> Result<(), CheatError> { read_process_memory(self.target_proc, address, buffer, size) }

    fn write_process_memory<T>(&self, address: LPVOID, buffer: *const T, size: usize) -> Result<(), CheatError> { write_process_memory(self.target_proc, address, buffer, size) }

    fn get_ptr(&self, offsets: Vec<u32>) -> Result<LPVOID, CheatError> {
        let mut ptr = BASE_ADDRESS + offsets[0];
        for offset in offsets[1..].iter() {
            let buffer: LPDWORD = &mut 0;
            self.read_process_memory(ptr as _, buffer as _, 4)?;
            unsafe { ptr = *buffer + offset; }
        }

        Ok(ptr as _)
    }

    fn hack_game_timer(&self, value: u32) -> Result<u32, CheatError> {
        //  exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x2C]
        let ptr = self.get_ptr(vec![0x14, 0x20, 0x2C])?;
        self.write_process_memory(ptr, &value as *const u32, 4)?;

        let time: LPDWORD = &mut 0;
        self.read_process_memory(ptr, time as _, 4)?;

        unsafe { Ok(*time) }
    }

    fn hack_chance(&self, value: u32) -> Result<u32, CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x3C]
        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x3C])?;
        self.write_process_memory(ptr, &value as *const u32, 4)?;

        let chance: LPDWORD = &mut 0;
        self.read_process_memory(ptr, chance as _, 4)?;

        unsafe { Ok(*chance) }
    }

    fn hack_tip(&self, value: u32) -> Result<u32, CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x38]
        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x38])?;
        self.write_process_memory(ptr, &value as *const u32, 4)?;

        let tip: LPDWORD = &mut 0;
        self.read_process_memory(ptr, tip as _, 4)?;

        unsafe { Ok(*tip) }
    }

    fn hack_score(&self, value: u32) -> Result<u32, CheatError> {
        // exp = [[0x004C0E2C + 0x14] + 0x3F34]
        let ptr = self.get_ptr(vec![0x14, 0x3F34])?;
        self.write_process_memory(ptr, &value as *const u32, 4)?;

        let score: LPDWORD = &mut 0;
        self.read_process_memory(ptr, score as _, 4)?;

        unsafe { Ok(*score) }
    }

    fn hack_combo_timer(&self, value: f32) -> Result<f32, CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x30]
        let ptr = self.get_ptr(vec![0x14, 0x20, 0x30])?;
        self.write_process_memory(ptr, &value as *const f32, 4)?;

        let time: LPDWORD = &mut 0;
        self.read_process_memory(ptr, time as _, 4)?;

        unsafe { Ok(*(time as *const f32)) }
    }

    fn hack_cells(&self) -> Result<[u32; 400], CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x180]
        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x180])?;

        let mut cells = [0; 400];
        self.read_process_memory(ptr, cells.as_mut_ptr() as _, 1600)?;

        Ok(cells)
    }

    fn option(&mut self) -> Result<(), CheatError> {
        // --- std ---
        use std::io::{Write, stdin, stdout};

        print!("1.hack game time\n2.hack chance\n3.hack tip\n4.hack score\n5.hack combo time\n6.hack cells\n7.exit\nfunction: ");
        stdout().flush().unwrap();
        let mut function = String::new();
        stdin().read_line(&mut function).unwrap();

        self.read_process_memory(BASE_ADDRESS as _, 0 as _, 0)?;

        match function.trim() {
            "1" => {
                let time = self.hack_game_timer(200)?;
                println!("time: {}", time);
            }
            "2" => {
                let chance = self.hack_chance(200)?;
                println!("chance: {}", chance);
            }
            "3" => {
                let tip = self.hack_tip(200)?;
                println!("tip: {}", tip);
            }
            "4" => {
                let score = self.hack_score(2000)?;
                println!("score: {}", score);
            }
            "5" => {
                let combo_time = self.hack_combo_timer(1000.)?;
                println!("combo_time: {}", combo_time);
            }
            "6" => {
                let cells = {
                    let mut col = vec![];
                    for chunk in self.hack_cells()?.chunks(50) {
                        let mut row = vec![];
                        for &x in chunk.iter() { if x == 0xffffffff { continue; } else { row.push(x); } }
                        if !row.is_empty() { col.push(row); }
                    }

                    col
                };

                println!("cells: [");
                for row in cells {
                    print!("    ");
                    for cell in row { print!("{:2}, ", cell); }
                    println!();
                }
                println!("]");
            }
            _ => return Err(CheatError::Exit)
        }

        Ok(())
    }
}

pub fn run() {
    // --- std ---
    use std::{
        thread::sleep,
        time::Duration
    };

    let mut cheat = Cheat::new();
    'inject: loop {
        if let Err(e) = cheat.inject() { println!("{:?}", e); } else {
            loop {
                if let Err(e) = cheat.option() {
                    cheat.exit();

                    println!("{:?}", e);
                    match e {
                        CheatError::Exit => break 'inject,
                        _ => break
                    }
                }
            }
        }

        sleep(Duration::from_millis(500));
    }
}
