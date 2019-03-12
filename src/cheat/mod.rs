mod remote;
mod util;

// --- external ---
use winapi::{
    shared::minwindef::{LPDWORD, LPVOID},
    um::winnt::HANDLE,
};
// --- custom ---
use self::{
    remote::*,
    util::*,
};

const BASE_ADDRESS: u32 = 0x4C0E2C;

struct Cheat {
    process: Processes,
    target_proc: HANDLE,
    remote_f32: LPVOID,
    remote_info: (LPVOID, RemoteInfo),
    remote_procs: [LPVOID; 6],
}

impl Cheat {
    fn new() -> Cheat {
        Cheat {
            process: Processes::new().add("user32.dll", "MessageBoxW"),
            target_proc: 0 as _,
            remote_f32: 0 as _,
            remote_info: (0 as _, RemoteInfo::new()),
            remote_procs: [0 as _; 6],
        }
    }

    fn exit(&mut self) {
        // --- external ---
        use winapi::um::{
            handleapi::CloseHandle,
            memoryapi::VirtualFree,
            winnt::MEM_RELEASE,
        };

        self.target_proc = 0 as _;
        self.remote_procs = [0 as _; 6];

        unsafe {
            VirtualFree(self.remote_f32, 0 as _, MEM_RELEASE);
            VirtualFree(self.remote_info.0, 0 as _, MEM_RELEASE);
            for &address in self.remote_procs.iter() { VirtualFree(address, 0 as _, MEM_RELEASE); }
            CloseHandle(self.target_proc);
        }
    }

    fn get_ptr(&self, offsets: Vec<u32>) -> Result<LPVOID, CheatError> {
        let mut ptr = BASE_ADDRESS + offsets[0];
        for offset in offsets[1..].iter() {
            let buffer: LPDWORD = &mut 0;
            read_process_memory(self.target_proc, ptr as _, buffer as _, 4)?;
            unsafe { ptr = *buffer + offset; }
        }

        Ok(ptr as _)
    }

    fn hack_game_timer(&mut self, value: u32) -> Result<(), CheatError> {
        //  exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x2C]
//        let ptr = self.get_ptr(vec![0x14, 0x20, 0x2C])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.show_message(&format!("Game timer = {}", value))?;
        create_remote_thread(self.target_proc, self.remote_procs[1], value as _)
    }

    fn hack_chance(&mut self, value: u32) -> Result<(), CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x3C]
//        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x3C])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.show_message(&format!("Chance = {}", value))?;
        create_remote_thread(self.target_proc, self.remote_procs[2], value as _)
    }

    fn hack_tip(&mut self, value: u32) -> Result<(), CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x38]
//        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x38])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.show_message(&format!("Tip = {}", value))?;
        create_remote_thread(self.target_proc, self.remote_procs[3], value as _)
    }

    fn hack_score(&mut self, value: u32) -> Result<(), CheatError> {
        // exp = [[0x004C0E2C + 0x14] + 0x3F34]
//        let ptr = self.get_ptr(vec![0x14, 0x3F34])?;
//        self.write_process_memory(ptr, &value as *const u32, 4)?;

        self.show_message(&format!("Score = {}", value))?;
        create_remote_thread(self.target_proc, self.remote_procs[4], value as _)
    }

    fn hack_combo_timer(&mut self, value: f32) -> Result<(), CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x30]
//        let ptr = self.get_ptr(vec![0x14, 0x20, 0x30])?;
//        self.write_process_memory(ptr, &value as *const f32, 4)?;

        self.show_message(&format!("Combo time = {}", value))?;
        write_process_memory(self.target_proc, self.remote_f32, &value as *const f32, 4)?;
        create_remote_thread(self.target_proc, self.remote_procs[5], self.remote_f32)
    }

    fn hack_cells(&self) -> Result<[u32; 400], CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x180]
        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x180])?;

        let mut cells = [0; 400];
        read_process_memory(self.target_proc, ptr, cells.as_mut_ptr() as _, 1600)?;

        Ok(cells)
    }

    fn option(&mut self) -> Result<(), CheatError> {
        // --- std ---
        use std::io::{Write, stdin, stdout};

        print!("1.hack game time\n2.hack chance\n3.hack tip\n4.hack score\n5.hack combo time\n6.hack cells\npress any other key to exit\n$>: ");
        stdout().flush().unwrap();
        let mut function = String::new();
        stdin().read_line(&mut function).unwrap();

        read_process_memory(self.target_proc, BASE_ADDRESS as _, 0 as _, 0)?;

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
