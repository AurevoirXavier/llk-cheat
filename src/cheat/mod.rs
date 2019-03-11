mod util;

// --- std ---
use std::mem::size_of;
// --- external ---
use winapi::{
    shared::{
        windef::HWND,
        minwindef::{FARPROC, LPDWORD, LPVOID},
    },
    um::winnt::{HANDLE, LPCWSTR},
};
// --- custom ---
use self::util::*;

const BASE_ADDRESS: u32 = 0x4C0E2C;

type MessageBoxProc = unsafe extern "system" fn(HWND, LPCWSTR, LPCWSTR, u32) -> i32;
type RemoteThreadProc = unsafe extern "system" fn(LPVOID) -> u32;

struct RemoteParam {
    command: u8,
    value_u32: u32,
    value_f32: f32,
    message_box_w_proc: FARPROC,
    title: [u16; 100],
    content: [u16; 100],
}

struct Cheat {
    process: Processes,
    target_proc: HANDLE,
    remote_param: RemoteParam,
    remote_param_ptr: LPVOID,
    remote_proc_ptr: LPVOID,
}

impl RemoteParam {
    fn new() -> RemoteParam {
        RemoteParam {
            command: 0,
            value_u32: 0,
            value_f32: 0.,
            title: [0; 100],
            content: [0; 100],
            message_box_w_proc: 0 as _,
        }
    }

    fn command(&mut self, command: u8) -> &mut Self {
        self.command = command;
        self
    }

    fn value_u32(&mut self, value: u32) -> &mut Self {
        self.value_u32 = value;
        self
    }

    fn value_f32(&mut self, value: f32) -> &mut Self {
        self.value_f32 = value;
        self
    }

    fn message_box_proc(&mut self, ptr: FARPROC) -> &mut Self {
        self.message_box_w_proc = ptr;
        self
    }

    fn title(&mut self, s: &str) -> &mut Self {
        let mut slice = s.encode_utf16().collect::<Vec<u16>>();
        slice.resize(self.title.len(), 0);
        self.title.clone_from_slice(&slice);

        self
    }

    fn content(&mut self, s: &str) -> &mut Self {
        let mut slice = s.encode_utf16().collect::<Vec<u16>>();
        slice.resize(self.content.len(), 0);
        self.content.clone_from_slice(&slice);

        self
    }
}

unsafe extern "system" fn remote_thread_proc(l_param: LPVOID) -> u32 {
    let RemoteParam {
        command,
        value_u32,
        value_f32,
        ref title,
        ref content,
        ref message_box_w_proc,
    } = *(l_param as *const RemoteParam);
//    (*(message_box_w_proc as *const _ as *const MessageBoxProc))(0 as _, content as *const [u16] as _, title as *const [u16] as _, 0);
    match command {
        //  exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x2C]
        1 => asm! {r"
            mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
            mov ecx, dword ptr ds:[ecx + 0x20]
            mov dword ptr ds:[ecx + 0x2C], $0"
            :
            : "r"(value_u32)
            :
            : "volatile", "intel"
        },
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x3C]
        2 => asm! {r"
            mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
            mov ecx, dword ptr ds:[ecx + 0x3FB8]
            mov dword ptr ds:[ecx + 0x3C], $0"
            :
            : "r"(0)
            :
            : "volatile", "intel"
        },
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x38]
        3 => asm! {r"
            mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
            mov ecx, dword ptr ds:[ecx + 0x3FB8]
            mov dword ptr ds:[ecx + 0x38], $0"
            :
            : "r"(value_u32)
            :
            : "volatile", "intel"
        },
        // exp = [[0x004C0E2C + 0x14] + 0x3F34]
        4 => asm! {r"
            mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
            mov dword ptr ds:[ecx + 0x3F34], $0"
            :
            : "r"(value_u32)
            :
            : "volatile", "intel"
        },
        // exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x30]
        5 => (),
        _ => ()
    }

    0
}

impl Cheat {
    fn new() -> Cheat {
        Cheat {
            process: Processes::new().add("user32.dll", "MessageBoxW"),
            target_proc: 0 as _,
            remote_param: RemoteParam::new(),
            remote_param_ptr: 0 as _,
            remote_proc_ptr: 0 as _,
        }
    }

    fn inject(&mut self) -> Result<(), CheatError> {
        // --- external ---
        use winapi::um::{
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

            self.remote_param
                .message_box_proc(*self.process.0.get("MessageBoxW").unwrap())
                .title("Author: Xavier")
                .content("Inject succeed");

            if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else { self.write_process_memory(ptr, &self.remote_param as *const RemoteParam, size)?; }

            self.remote_param_ptr = ptr;
        }
        {
            let size = 0x1000;
            let ptr = unsafe { VirtualAllocEx(self.target_proc, 0 as _, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };
            if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else { self.write_process_memory(ptr, remote_thread_proc as *const RemoteThreadProc, size)?; }

            self.remote_proc_ptr = ptr;
        }

        self.create_remote_thread()
    }

    fn exit(&mut self) {
        // --- external ---
        use winapi::um::{
            handleapi::CloseHandle,
            memoryapi::VirtualFree,
            winnt::MEM_RELEASE,
        };

        self.target_proc = 0 as _;
        self.remote_param_ptr = 0 as _;
        self.remote_proc_ptr = 0 as _;

        unsafe {
            CloseHandle(self.target_proc);
            VirtualFree(self.remote_param_ptr, 0 as _, MEM_RELEASE);
            VirtualFree(self.remote_proc_ptr, 0 as _, MEM_RELEASE);
        }
    }

    fn get_ptr(&self, offsets: Vec<u32>) -> Result<LPVOID, CheatError> {
        let mut ptr = BASE_ADDRESS + offsets[0];
        for offset in offsets[1..].iter() {
            let buffer: LPDWORD = &mut 0;
            self.read_process_memory(ptr as _, buffer as _, 4)?;
            unsafe { ptr = *buffer + offset; }
        }

        Ok(ptr as _)
    }

    fn hack_game_timer(&mut self, value: u32) -> Result<(), CheatError> {
        //  exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x2C]
//        let ptr = self.get_ptr(vec![0x14, 0x20, 0x2C])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.remote_param
            .command(1)
            .value_u32(value)
            .title("Hack")
            .content(&format!("Game timer = {}", value));
        self.write_process_memory(self.remote_param_ptr, &self.remote_param as _, size_of::<RemoteParam>())?;
        self.create_remote_thread()?;

        Ok(())
    }

    fn hack_chance(&mut self, value: u32) -> Result<(), CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x3C]
//        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x3C])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.remote_param
            .command(2)
            .value_u32(value)
            .title("Hack")
            .content(&format!("Chance = {}", value));
        self.write_process_memory(self.remote_param_ptr, &self.remote_param as _, size_of::<RemoteParam>())?;
        self.create_remote_thread()?;

        Ok(())
    }

    fn hack_tip(&mut self, value: u32) -> Result<(), CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x38]
//        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x38])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.remote_param
            .command(3)
            .value_u32(value)
            .title("Hack")
            .content(&format!("Tip = {}", value));
        self.write_process_memory(self.remote_param_ptr, &self.remote_param as _, size_of::<RemoteParam>())?;
        self.create_remote_thread()?;

        Ok(())
    }

    fn hack_score(&mut self, value: u32) -> Result<(), CheatError> {
        // exp = [[0x004C0E2C + 0x14] + 0x3F34]
//        let ptr = self.get_ptr(vec![0x14, 0x3F34])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.remote_param
            .command(4)
            .value_u32(value)
            .title("Hack")
            .content(&format!("Score = {}", value));
        self.write_process_memory(self.remote_param_ptr, &self.remote_param as _, size_of::<RemoteParam>())?;
        self.create_remote_thread()?;

        Ok(())
    }

    fn hack_combo_timer(&mut self, value: f32) -> Result<(), CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x30]
//        let ptr = self.get_ptr(vec![0x14, 0x20, 0x30])?;
//        self.write_process_memory(ptr, &value as *const f32, 4)?;

        self.remote_param
            .command(5)
            .value_f32(value)
            .title("Hack")
            .content(&format!("Combo timer = {}", value));
        self.write_process_memory(self.remote_param_ptr, &self.remote_param as _, size_of::<RemoteParam>())?;
        self.create_remote_thread()?;

        Ok(())
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

        print!("1.hack game time\n2.hack chance\n3.hack tip\n4.hack score\n5.hack combo time\n6.hack cells\npress any other key to exit\n$>: ");
        stdout().flush().unwrap();
        let mut function = String::new();
        stdin().read_line(&mut function).unwrap();

        self.read_process_memory(BASE_ADDRESS as _, 0 as _, 0)?;

        match function.trim() {
            "1" => self.hack_game_timer(2000)?,
            "2" => self.hack_chance(2000)?,
            "3" => self.hack_tip(2000)?,
            "4" => self.hack_score(2000)?,
            "5" => self.hack_combo_timer(2000.)?,
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
        time::Duration,
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
