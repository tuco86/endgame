use std::mem::size_of;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromWindow,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_TERMINATE, QueryFullProcessImageNameW, TerminateProcess,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GWL_EXSTYLE, GetForegroundWindow, GetWindowLongW, GetWindowRect,
    GetWindowThreadProcessId, IsWindowVisible, WS_EX_TOOLWINDOW,
};

use crate::config::Config;
use windows_sys::Win32::Foundation::LUID;
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

pub fn enable_debug_privilege() -> bool {
    unsafe {
        let mut token: HANDLE = 0 as HANDLE;
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return false;
        }
        let mut luid: LUID = std::mem::zeroed();
        let name: Vec<u16> = "SeDebugPrivilege".encode_utf16().chain([0]).collect();
        if LookupPrivilegeValueW(null_mut(), name.as_ptr(), &mut luid) == 0 {
            CloseHandle(token);
            return false;
        }
        const SE_PRIVILEGE_ENABLED_FLAG: u32 = 0x00000002;
        let la = LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED_FLAG,
        };
        let tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [la],
        };
        let res = AdjustTokenPrivileges(token, 0, &tp, 0, null_mut(), null_mut());
        let success = res != 0 && GetLastError() == 0;
        CloseHandle(token);
        success
    }
}

pub struct TargetProcess {
    pub pid: u32,
    pub name: String,
}

pub fn find_target(config: &Config) -> Option<TargetProcess> {
    if let Some(tp) = find_whitelist(config) {
        log::info!("Whitelist match: {} (PID {})", tp.name, tp.pid);
        return Some(tp);
    }
    if let Some(tp) = foreground_process(config) {
        log::info!("Foreground match: {} (PID {})", tp.name, tp.pid);
        return Some(tp);
    }
    if let Some(tp) = fullscreen_fallback(config) {
        log::info!("Fullscreen fallback match: {} (PID {})", tp.name, tp.pid);
        return Some(tp);
    }
    None
}

fn find_whitelist(config: &Config) -> Option<TargetProcess> {
    if config.whitelist.is_empty() {
        return None;
    }
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == -1i32 as isize as _ {
        return None;
    } // INVALID_HANDLE_VALUE
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut result = unsafe { Process32FirstW(snapshot, &mut entry) };
    while result != 0 {
        if let Some(name) = wide_to_string(&entry.szExeFile) {
            let lname = name.to_lowercase();
            if config.whitelist.iter().any(|w| w.to_lowercase() == lname)
                && !is_blacklisted(&lname, config)
            {
                unsafe {
                    CloseHandle(snapshot);
                }
                return Some(TargetProcess {
                    pid: entry.th32ProcessID,
                    name,
                });
            }
        }
        result = unsafe { Process32NextW(snapshot, &mut entry) };
    }
    unsafe {
        CloseHandle(snapshot);
    }
    None
}

fn foreground_process(config: &Config) -> Option<TargetProcess> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return None;
    }
    let pid = window_pid(hwnd)?;
    process_name(pid).and_then(|name| {
        let lname = name.to_lowercase();
        if is_blacklisted(&lname, config) {
            None
        } else {
            Some(TargetProcess { pid, name })
        }
    })
}

fn fullscreen_fallback(config: &Config) -> Option<TargetProcess> {
    // Enumerate windows and choose the one covering its monitor fully
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
        unsafe {
            let data = &mut *(lparam as *mut (HWND, i64));
            if IsWindowVisible(hwnd) == 0 {
                return 1;
            }
            let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            if (ex & WS_EX_TOOLWINDOW) != 0 {
                return 1;
            }
            let mut rect: RECT = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return 1;
            }
            let monitor: HMONITOR = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
            if monitor.is_null() {
                return 1;
            }
            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut mi as *mut MONITORINFO) == 0 {
                return 1;
            }
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            let mw = mi.rcMonitor.right - mi.rcMonitor.left;
            let mh = mi.rcMonitor.bottom - mi.rcMonitor.top;
            if w == mw && h == mh {
                let area = (w as i64) * (h as i64);
                if area > data.1 {
                    data.0 = hwnd;
                    data.1 = area;
                }
            }
            1
        }
    }
    let mut tuple: (HWND, i64) = (null_mut(), 0);
    let ptr = &mut tuple as *mut _ as LPARAM;
    let _ = unsafe { EnumWindows(Some(enum_proc), ptr) };
    let (best_hwnd, best_area) = tuple;
    if best_hwnd.is_null() || best_area == 0 {
        return None;
    }
    let pid = window_pid(best_hwnd)?;
    process_name(pid).and_then(|name| {
        let lname = name.to_lowercase();
        if is_blacklisted(&lname, config) {
            None
        } else {
            Some(TargetProcess { pid, name })
        }
    })
}

fn is_blacklisted(name_lower: &str, config: &Config) -> bool {
    config
        .blacklist
        .iter()
        .any(|b| b.to_lowercase() == name_lower)
}

fn window_pid(hwnd: HWND) -> Option<u32> {
    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if pid == 0 { None } else { Some(pid) }
}

fn process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(0x0410 /* QUERY_LIMITED_INFO */, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;
        let success = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        if success == 0 {
            CloseHandle(handle);
            return None;
        }
        CloseHandle(handle);
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        // extract file name
        let name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
        Some(name)
    }
}

pub fn terminate(pid: u32) -> bool {
    unsafe {
        let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if h.is_null() {
            log::error!("OpenProcess failed for PID {}", pid);
            return false;
        }
        let ok = TerminateProcess(h, 1) != 0;
        CloseHandle(h);
        if !ok {
            log::error!("TerminateProcess failed for PID {}", pid);
        }
        ok
    }
}

fn wide_to_string(buf: &[u16]) -> Option<String> {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    if len == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..len]))
}
