// --- reexport ---
pub use self::{
    param::RemoteInfo,
    proc::*,
};

// --- std ---
use std::mem::size_of;
// --- external ---
use winapi::{
    shared::minwindef::LPVOID,
    um::{
        memoryapi::VirtualAllocEx,
        winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE, HANDLE},
    },
};
// --- custom ---
use super::{
    Cheat,
    util::*,
};
use self::{param::MessageBoxProc};

mod proc;
mod param;

impl Cheat {
    fn open_process(&mut self) -> Result<(), CheatError> {
        self.target_proc = open_process(get_window_thread_process_id(find_window()?)?)?;
        Ok(())
    }

    fn inject_info(&mut self) -> Result<(), CheatError> {
        let size = size_of::<RemoteInfo>();
        let ptr = unsafe { VirtualAllocEx(self.target_proc, 0 as _, size, MEM_COMMIT, PAGE_READWRITE) };
        if ptr.is_null() { Err(CheatError::VirtualAllocError) } else {
            self.remote_info.0 = ptr;
            self.remote_info.1
                .message_box_proc(*self.process.0.get("MessageBoxW").unwrap())
                .title("Author: Xavier")
                .content("Inject succeed");

            write_process_memory(self.target_proc, ptr, &self.remote_info.1 as *const RemoteInfo, size)?;
            Ok(())
        }
    }

    fn inject_f32(&mut self) -> Result<(), CheatError>  {
        let ptr = unsafe { VirtualAllocEx(self.target_proc, 0 as _, 4, MEM_COMMIT, PAGE_READWRITE) };
        if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else {
            self.remote_f32 = ptr;
            Ok(())
        }
    }

    pub fn inject(&mut self) -> Result<(), CheatError> {
        self.open_process()?;
        self.inject_info()?;
        self.inject_f32()?;
        for (i, &proc) in [
            info,
            game_timer,
            chance,
            tip,
            score,
            combo_time
        ].iter().enumerate() { self.remote_procs[i] = inject_proc(self.target_proc, proc, 256)?; }

        create_remote_thread(self.target_proc, self.remote_procs[0], self.remote_info.0)
    }

    pub fn show_message(&mut self, message: &str) -> Result<(), CheatError> {
        self.remote_info.1
            .title("Hack")
            .content(message);
        write_process_memory(self.target_proc, self.remote_info.0, &self.remote_info.1 as _, size_of::<RemoteInfo>())?;
        create_remote_thread(self.target_proc, self.remote_procs[0], self.remote_info.0)
    }
}

fn inject_proc(handle: HANDLE, proc: RemoteProc, size: usize) -> Result<LPVOID, CheatError> {
    let ptr = unsafe { VirtualAllocEx(handle, 0 as _, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };
    if ptr.is_null() { return Err(CheatError::VirtualAllocError); } else {
        write_process_memory(handle, ptr, proc as *const RemoteProc, size)?;
        Ok(ptr)
    }
}
