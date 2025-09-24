use std::sync::mpsc::{self, Receiver};
use std::thread;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, VK_END};
use windows_sys::Win32::UI::WindowsAndMessaging::{MSG, WM_HOTKEY};

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
}

pub const HOTKEY_ID: i32 = 1; // arbitrary id

pub fn spawn_hotkey_listener() -> Receiver<()> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || unsafe {
        if RegisterHotKey(0 as HWND, HOTKEY_ID, MOD_CONTROL | MOD_ALT, VK_END as u32) == 0 {
            log::error!("Failed to register hotkey Ctrl+Alt+End");
            return;
        }
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg as *mut MSG, 0 as HWND, 0, 0) != 0 {
            if msg.message == WM_HOTKEY && msg.wParam == HOTKEY_ID as usize {
                let _ = tx.send(());
            }
        }
        let _ = UnregisterHotKey(0 as HWND, HOTKEY_ID);
    });
    rx
}
