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
            Self::get_windows_win32()
        }

        #[cfg(target_os = "linux")]
        {
            Self::get_windows_linux()
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Vec::new()
        }
    }

    #[cfg(target_os = "windows")]
    fn get_windows_win32() -> Vec<WindowEntry> {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextW, IsWindowVisible,
        };

        let mut windows_list = Vec::new();

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let list = &mut *(lparam.0 as *mut Vec<WindowEntry>);

            if IsWindowVisible(hwnd).as_bool() {
                let mut title_buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut title_buf);
                if len > 0 {
                    let title = String::from_utf16_lossy(&title_buf[..len as usize]).trim().to_string();
                    // Ignore common hidden overlay titles
                    if !title.is_empty() 
                        && title != "Finder" 
                        && title != "Program Manager" 
                        && title != "Settings" 
                        && !title.starts_with("Default IME")
                        && !title.starts_with("MSCTFIME")
                    {
                        list.push(WindowEntry {
                            title: title.clone(),
                            description: format!("Active Window (HWND: 0x{:X})", hwnd.0 as usize),
                            hwnd: hwnd.0 as isize,
                            category: "Windows".into(),
                            icon: "🪟".into(),
                        });
                    }
                }
            }
            BOOL(1)
        }

        unsafe {
            let _ = EnumWindows(
                Some(enum_proc),
                LPARAM(&mut windows_list as *mut _ as isize),
            );
        }

        windows_list
    }

    #[cfg(target_os = "linux")]
    fn get_windows_linux() -> Vec<WindowEntry> {
        let mut list = Vec::new();
        // Query wmctrl -l
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
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{
                SetForegroundWindow, ShowWindow, SW_RESTORE,
            };

            unsafe {
                let h = HWND(hwnd as *mut std::ffi::c_void);
                ShowWindow(h, SW_RESTORE);
                SetForegroundWindow(h).as_bool()
            }
        }

        #[cfg(target_os = "linux")]
        {
            let id_hex = format!("0x{:x}", hwnd);
            std::process::Command::new("wmctrl")
                .args(["-i", "-a", &id_hex])
                .spawn()
                .is_ok()
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            let _ = hwnd;
            false
        }
    }
}
