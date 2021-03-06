mod llk_algorithm;
mod remote;
mod util;

// --- std ---
use std::{
    thread::sleep,
    time::Duration,
};
// --- external ---
use winapi::{
    shared::{
        minwindef::{LPDWORD, LPVOID},
        windef::HWND,
    },
    um::winnt::HANDLE,
};
// --- custom ---
use self::{
    remote::*,
    util::*,
};

const BASE_ADDRESS: u32 = 0x4C0E2C;

struct Cells(Vec<Vec<u8>>);

struct Cheat {
    process: Processes,
    target_window: HWND,
    target_proc: HANDLE,
    remote_f32: LPVOID,
    remote_info: (LPVOID, RemoteInfo),
    remote_procs: [LPVOID; 6],
}

impl Cheat {
    fn new() -> Cheat {
        Cheat {
            process: Processes::new().add("user32.dll", "MessageBoxW"),
            target_window: 0 as _,
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

    fn hack_cells(&self) -> Result<Cells, CheatError> {
        // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x180]
        let ptr = self.get_ptr(vec![0x14, 0x3FB8, 0x180])?;

        let mut buffer = [0u32; 400];
        read_process_memory(self.target_proc, ptr, buffer.as_mut_ptr() as _, 1600)?;

        let start = buffer[0..50]
            .splitn(2, |&x| x != 0xffffffff)
            .next()
            .unwrap()
            .len() - 1;
        let end = 51 - buffer[0..50]
            .rsplitn(2, |&x| x != 0xffffffff)
            .next()
            .unwrap()
            .len();

        let cells: Vec<Vec<u8>> = buffer.chunks(50)
            .filter(|row| row.iter().any(|&v| v != 0xffffffff))
            .map(|chunk|
                chunk[start..end].iter()
                    .map(|&v| v as u8)
                    .collect())
            .collect();

        let mut cells_with_edge = vec![vec![255; cells[0].len()]];
        for row in cells { cells_with_edge.push(row); }

        cells_with_edge.push(vec![255; cells_with_edge[0].len()]);
        Ok(Cells(cells_with_edge))
    }

    fn eliminate_cells(&self, w: u32, h: u32, x_edge: u32, y_edge: u32, scale: f32) -> Result<(), CheatError> {
        // --- external ---
        use winapi::um::winuser::{WM_LBUTTONDOWN, WM_LBUTTONUP, SendMessageA};

        let (x_edge, y_edge) = ((x_edge as f32 / scale) as u32, (y_edge as f32 / scale) as u32);
        for (x, y) in self.hack_cells()?.solve() {
            let x = ((x - 1) as u32 * w + x_edge + w / 2) as f32;
            let y = ((y - 1) as u32 * h + y_edge + h / 2) as f32;
            let xy = (((y / scale) as isize) << 16) + (x / scale) as isize;

            unsafe {
                SendMessageA(self.target_window, WM_LBUTTONDOWN, 1, xy);
                SendMessageA(self.target_window, WM_LBUTTONUP, 0, xy);
            }

            sleep(Duration::from_millis(1));
        }

        Ok(())
    }

    fn option(&mut self) -> Result<(), CheatError> {
        // --- std ---
        use std::io::{Write, stdin, stdout};

        print!("1.hack game time\n2.hack chance\n3.hack tip\n4.hack score\n5.hack combo time\n6.hack cells\n7.eliminate cells\npress any other key to exit\n$>: ");
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
                let cells = self.hack_cells()?;
                println!("format: [");
                for (y, row) in cells.0.into_iter().enumerate() {
                    print!("    ");
                    for (x, v) in row.into_iter().enumerate() { print!("({:3}, {:2}, {:2}), ", v, x, y); }
                    println!();
                }
                println!("]");
            }
            "7" => self.eliminate_cells(58, 64, 40, 88, 1.25)?,
            _ => return Err(CheatError::Exit)
        }

        Ok(())
    }
}

pub fn run() {
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
