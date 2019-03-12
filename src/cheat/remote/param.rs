// --- external ---
use winapi::{
    shared::{
        windef::HWND,
        minwindef::FARPROC,
    },
    um::winnt::LPCWSTR,
};

pub type MessageBoxProc = unsafe extern "system" fn(HWND, LPCWSTR, LPCWSTR, u32) -> i32;

pub struct RemoteInfo {
    pub message_box_w: FARPROC,
    pub title: [u16; 100],
    pub content: [u16; 100],
}

impl RemoteInfo {
    pub fn new() -> RemoteInfo {
        RemoteInfo {
            title: [0; 100],
            content: [0; 100],
            message_box_w: 0 as _,
        }
    }

    pub fn message_box_proc(&mut self, ptr: FARPROC) -> &mut Self {
        self.message_box_w = ptr;
        self
    }

    pub fn title(&mut self, s: &str) -> &mut Self {
        let mut slice = s.encode_utf16().collect::<Vec<u16>>();
        slice.resize(self.title.len(), 0);
        self.title.clone_from_slice(&slice);

        self
    }

    pub fn content(&mut self, s: &str) -> &mut Self {
        let mut slice = s.encode_utf16().collect::<Vec<u16>>();
        slice.resize(self.content.len(), 0);
        self.content.clone_from_slice(&slice);

        self
    }
}
