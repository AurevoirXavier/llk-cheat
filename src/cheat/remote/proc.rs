// --- external ---
use winapi::shared::minwindef::LPVOID;
// --- custom ---
use super::{MessageBoxProc, RemoteInfo};

pub type RemoteProc = unsafe extern "system" fn(LPVOID) -> u32;

pub unsafe extern "system" fn info(l_param: LPVOID) -> u32 {
    let RemoteInfo {
        ref title,
        ref content,
        ref message_box_w,
    } = *(l_param as *const RemoteInfo);
    (*(message_box_w as *const _ as *const MessageBoxProc))(0 as _, content as *const [u16] as _, title as *const [u16] as _, 0);

    0
}

pub unsafe extern "system" fn game_timer(value: LPVOID) -> u32 {
    //  exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x2C]
    asm! {r"
            mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
            mov ecx, dword ptr ds:[ecx + 0x20]
            mov dword ptr ds:[ecx + 0x2C], $0"
            :
            : "r"(value as u32)
            :
            : "volatile", "intel"
    }

    0
}

pub unsafe extern "system" fn chance(value: LPVOID) -> u32 {
    // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x3C]
    asm! {r"
        mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
        mov ecx, dword ptr ds:[ecx + 0x3FB8]
        mov dword ptr ds:[ecx + 0x3C], $0"
        :
        : "r"(value as u32)
        :
        : "volatile", "intel"
    }

    0
}

pub unsafe extern "system" fn tip(value: LPVOID) -> u32 {
    // exp = [[[0x4C0E2C + 0x14] + 0x3FB8] + 0x38]
    asm! {r"
        mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
        mov ecx, dword ptr ds:[ecx + 0x3FB8]
        mov dword ptr ds:[ecx + 0x38], $0"
        :
        : "r"(value as u32)
        :
        : "volatile", "intel"
    }

    0
}

pub unsafe extern "system" fn score(value: LPVOID) -> u32 {
    // exp = [[0x004C0E2C + 0x14] + 0x3F34]
    asm! {r"
        mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
        mov dword ptr ds:[ecx + 0x3F34], $0"
        :
        : "r"(value as u32)
        :
        : "volatile", "intel"
    }

    0
}

pub unsafe extern "system" fn combo_time(value: LPVOID) -> u32 {
    // exp = [[[0x4C0E2C + 0x14] + 0x20] + 0x30]
    asm! {r"
        mov ecx, dword ptr ds:[0x4C0E2C + 0x14]
        mov ecx, dword ptr ds:[ecx + 0x20]
        mov dword ptr ds:[ecx + 0x30], $0"
        :
        : "r"(*(value as *const f32))
        :
        : "volatile", "intel"
    }

    0
}
