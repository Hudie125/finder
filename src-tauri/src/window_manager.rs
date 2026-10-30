use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowEntry {
    pub title: String,
    pub description: String,
    pub hwnd: isize,
    pub category: String,
    pub icon: String,
}

pub struct WindowManager;

impl WindowManager {
    pub fn get_running_windows() -> Vec<WindowEntry> {
        #[cfg(target_os = "windows")]
        {
            win32::get_running_windows()
        }

        #[cfg(target_os = "linux")]
        {
            linux::get_running_windows()
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Vec::new()
        }
    }

    pub fn focus_window(hwnd: isize) -> bool {
        #[cfg(target_os = "windows")]
        {
            win32::focus_window(hwnd)
        }

        #[cfg(target_os = "linux")]
        {
            linux::focus_window(hwnd)
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            let _ = hwnd;
            false
        }
    }
}

#[cfg(target_os = "windows")]
mod win32 {
    use super::WindowEntry;

    extern "system" {
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn GetWindowTextW(hwnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
        fn EnumWindows(lpEnumFunc: unsafe extern "system" fn(isize, isize) -> i32, lParam: isize) -> i32;
        fn SetForegroundWindow(hwnd: isize) -> i32;
        fn ShowWindow(hwnd: isize, nCmdShow: i32) -> i32;
    }

    unsafe extern "system" fn enum_proc(hwnd: isize, lparam: isize) -> i32 {
        let list = &mut *(lparam as *mut Vec<WindowEntry>);
        if IsWindowVisible(hwnd) != 0 {
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            if len > 0 {
                let title = String::from_utf16_lossy(&title_buf[..len as usize]).trim().to_string();
                if !title.is_empty() 
                    && title != "Finder" 
                    && title != "Program Manager" 
                    && title != "Settings" 
                    && !title.starts_with("Default IME")
                    && !title.starts_with("MSCTFIME")
                {
                    list.push(WindowEntry {
                        title: title.clone(),
                        description: format!("Active Window (HWND: 0x{:X})", hwnd as usize),
                        hwnd,
                        category: "Windows".into(),
                        icon: "🪟".into(),
                    });
                }
            }
        }
        1
    }

    pub fn get_running_windows() -> Vec<WindowEntry> {
        let mut list = Vec::new();
        unsafe {
            let _ = EnumWindows(enum_proc, &mut list as *mut _ as isize);
        }
        list
    }

    pub fn focus_window(hwnd: isize) -> bool {
        unsafe {
            ShowWindow(hwnd, 9); // 9 = SW_RESTORE
            SetForegroundWindow(hwnd) != 0
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::WindowEntry;

    pub fn get_running_windows() -> Vec<WindowEntry> {
        let mut list = Vec::new();
        if let Ok(output) = std::process::Command::new("wmctrl").arg("-l").output() {
            if let Ok(text) = String::from_utf8(output.stdout) {
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let id_str = parts[0];
                        let title = parts[3..].join(" ");
                        if !title.is_empty() && title != "Finder" {
                            let hwnd = isize::from_str_radix(id_str.trim_start_matches("0x"), 16).unwrap_or(0);
                            list.push(WindowEntry {
                                title: title.clone(),
                                description: format!("Window ID: {}", id_str),
                                hwnd,
                                category: "Windows".into(),
                                icon: "🪟".into(),
                            });
                        }
                    }
                }
            }
        }
        list
    }

    pub fn focus_window(hwnd: isize) -> bool {
        let id_hex = format!("0x{:x}", hwnd);
        std::process::Command::new("wmctrl")
            .args(["-i", "-a", &id_hex])
            .spawn()
            .is_ok()
    }
}
