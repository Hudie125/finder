// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_scanner;
mod commands;
mod file_search;
mod storage;
mod system_monitor;
mod weather;
mod window_manager;

use app_scanner::{AppEntry, AppScanner};
use commands::CommandRunner;
use file_search::FileSearchEngine;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use storage::{NoteItem, StorageManager};
use system_monitor::{HardwareMonitor, SystemTelemetry};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};
use weather::{WeatherData, WeatherService};
use window_manager::WindowManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    pub title: String,
    pub description: String,
    pub path: Option<String>,
    pub cmd: Option<String>,
    pub hwnd: Option<isize>,
    pub category: String,
    pub icon: String,
    pub snippet: Option<String>,
}

struct AppState {
    monitor: HardwareMonitor,
    cached_apps: Vec<AppEntry>,
    is_visible: Arc<AtomicBool>,
}

// ==========================================================================
// Tauri Commands
// ==========================================================================

#[tauri::command]
async fn search(
    query: String,
    category: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<UnifiedSearchResult>, String> {
    let q = query.trim().to_lowercase();
    let mut results = Vec::new();

    // 1. Commands Prefix ">"
    if q.starts_with('>') || category == "commands" {
        let cmd_query = q.trim_start_matches('>').trim();
        for cmd in CommandRunner::get_all_commands() {
            if cmd_query.is_empty()
                || cmd.title.to_lowercase().contains(cmd_query)
                || cmd.cmd.to_lowercase().contains(cmd_query)
            {
                results.push(UnifiedSearchResult {
                    title: cmd.title,
                    description: cmd.description,
                    path: None,
                    cmd: Some(cmd.cmd),
                    hwnd: None,
                    category: "Commands".into(),
                    icon: cmd.icon,
                    snippet: None,
                });
            }
        }
        if q.starts_with('>') {
            return Ok(results);
        }
    }

    // 2. Apps Search
    if category == "all" || category == "apps" {
        for app in &state.cached_apps {
            if q.is_empty()
                || app.title.to_lowercase().contains(&q)
                || app.description.to_lowercase().contains(&q)
            {
                results.push(UnifiedSearchResult {
                    title: app.title.clone(),
                    description: app.description.clone(),
                    path: Some(app.path.clone()),
                    cmd: Some(app.path.clone()),
                    hwnd: None,
                    category: app.category.clone(),
                    icon: app.icon.clone(),
                    snippet: None,
                });
                if results.len() >= 8 && !q.is_empty() {
                    break;
                }
            }
        }
    }

    // 3. Running Windows Search
    if category == "all" || category == "windows" {
        for win in WindowManager::get_running_windows() {
            if q.is_empty() || win.title.to_lowercase().contains(&q) {
                results.push(UnifiedSearchResult {
                    title: win.title,
                    description: win.description,
                    path: None,
                    cmd: None,
                    hwnd: Some(win.hwnd),
                    category: "Windows".into(),
                    icon: win.icon,
                    snippet: None,
                });
                if results.len() >= 12 && !q.is_empty() {
                    break;
                }
            }
        }
    }

    // 4. File Search & Content Grep
    if (category == "all" || category == "files") && q.len() >= 2 {
        let file_matches = FileSearchEngine::search_files(&query, 8);
        for f in file_matches {
            results.push(UnifiedSearchResult {
                title: f.title,
                description: f.description.clone(),
                path: Some(f.path),
                cmd: None,
                hwnd: None,
                category: "Files".into(),
                icon: f.icon,
                snippet: f.snippet,
            });
        }
    }

    // 5. Google Live Suggestion fallback
    if !q.is_empty() && results.len() < 4 {
        results.push(UnifiedSearchResult {
            title: format!("Search Google for \"{}\"", query),
            description: "Open in default web browser".into(),
            path: Some(format!("https://www.google.com/search?q={}", urlencoding(&query))),
            cmd: None,
            hwnd: None,
            category: "Web".into(),
            icon: "🌐".into(),
            snippet: None,
        });
    }

    Ok(results)
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
}

#[tauri::command]
fn launch_app(path: String, as_admin: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if as_admin {
            let script = format!("Start-Process -FilePath '{}' -Verb RunAs", path);
            let _ = Command::new("powershell")
                .args(["-NoProfile", "-Command", &script])
                .spawn();
        } else if path.starts_with("ms-") || path.starts_with("http") {
            let _ = Command::new("explorer.exe").arg(&path).spawn();
        } else {
            let _ = Command::new("cmd.exe").args(["/c", "start", "", &path]).spawn();
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if as_admin {
            let _ = Command::new("pkexec").arg(&path).spawn();
        } else {
            let _ = Command::new("xdg-open").arg(&path).spawn();
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (path, as_admin);
        Ok(())
    }
}

#[tauri::command]
fn focus_window(hwnd: isize) -> bool {
    WindowManager::focus_window(hwnd)
}

#[tauri::command]
fn execute_command(command: String) -> Result<String, String> {
    CommandRunner::execute(&command)
}

#[tauri::command]
fn get_system_telemetry(state: tauri::State<'_, Arc<AppState>>) -> SystemTelemetry {
    state.monitor.get_telemetry()
}

#[tauri::command]
async fn get_weather(city: String) -> Option<WeatherData> {
    WeatherService::fetch_weather(&city).await
}

#[tauri::command]
fn get_notes() -> Vec<NoteItem> {
    StorageManager::load_notes()
}

#[tauri::command]
fn save_note(text: String) {
    let mut notes = StorageManager::load_notes();
    notes.insert(
        0,
        NoteItem {
            text,
            timestamp: chrono::Local::now().to_rfc3339(),
        },
    );
    StorageManager::save_notes(&notes);
}

#[tauri::command]
fn delete_note(id: usize) {
    let mut notes = StorageManager::load_notes();
    if id < notes.len() {
        notes.remove(id);
        StorageManager::save_notes(&notes);
    }
}

#[tauri::command]
fn get_settings() -> serde_json::Value {
    StorageManager::load_settings()
}

#[tauri::command]
fn save_settings(settings: serde_json::Value) {
    StorageManager::save_settings(&settings);
    #[cfg(target_os = "windows")]
    if let Some(hotkey) = settings.get("hotkey").and_then(|v| v.as_str()) {
        global_hotkey::set_hotkey(hotkey);
    }
}

#[tauri::command]
fn hide_window(app: AppHandle, state: tauri::State<'_, Arc<AppState>>) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
        state.is_visible.store(false, Ordering::SeqCst);
    }
}

static LAST_SHOWN: std::sync::OnceLock<std::sync::Mutex<std::time::Instant>> = std::sync::OnceLock::new();

pub fn mark_shown() {
    let m = LAST_SHOWN.get_or_init(|| std::sync::Mutex::new(std::time::Instant::now()));
    if let Ok(mut lock) = m.lock() {
        *lock = std::time::Instant::now();
    }
}

pub fn should_ignore_blur() -> bool {
    let m = LAST_SHOWN.get_or_init(|| std::sync::Mutex::new(std::time::Instant::now()));
    if let Ok(lock) = m.lock() {
        lock.elapsed() < std::time::Duration::from_millis(350)
    } else {
        false
    }
}

#[cfg(target_os = "windows")]
pub fn force_foreground() {
    unsafe {
        extern "system" {
            fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
            fn GetForegroundWindow() -> isize;
            fn GetWindowThreadProcessId(hwnd: isize, lpdwProcessId: *mut u32) -> u32;
            fn GetCurrentThreadId() -> u32;
            fn AttachThreadInput(idAttach: u32, idAttachTo: u32, fAttach: i32) -> i32;
            fn SetForegroundWindow(hwnd: isize) -> i32;
            fn SetFocus(hwnd: isize) -> isize;
            fn BringWindowToTop(hwnd: isize) -> i32;
            fn ShowWindow(hwnd: isize, nCmdShow: i32) -> i32;
            fn SetWindowPos(hwnd: isize, hWndInsertAfter: isize, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
        }

        let title: Vec<u16> = "Finder\0".encode_utf16().collect();
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if hwnd == 0 {
            return;
        }

        const SW_RESTORE: i32 = 9;
        const HWND_TOPMOST: isize = -1;
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOSIZE: u32 = 0x0001;
        const SWP_SHOWWINDOW: u32 = 0x0040;

        let fg_hwnd = GetForegroundWindow();
        let fg_thread = GetWindowThreadProcessId(fg_hwnd, std::ptr::null_mut());
        let cur_thread = GetCurrentThreadId();

        if fg_thread != 0 && fg_thread != cur_thread {
            AttachThreadInput(cur_thread, fg_thread, 1);
        }

        ShowWindow(hwnd, SW_RESTORE);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
        SetFocus(hwnd);

        if fg_thread != 0 && fg_thread != cur_thread {
            AttachThreadInput(cur_thread, fg_thread, 0);
        }
    }
}

pub fn toggle_window_visibility(app: &AppHandle, is_visible: &AtomicBool) {
    if let Some(win) = app.get_webview_window("main") {
        let is_currently_visible = win.is_visible().unwrap_or(false);
        let is_focused = win.is_focused().unwrap_or(false);

        if is_currently_visible && is_focused {
            let _ = win.hide();
            is_visible.store(false, Ordering::SeqCst);
        } else {
            mark_shown();
            let _ = win.show();
            #[cfg(target_os = "windows")]
            force_foreground();
            let _ = win.set_focus();
            is_visible.store(true, Ordering::SeqCst);
            let _ = win.emit("focus-search", ());
        }
    }
}

#[cfg(target_os = "windows")]
mod single_instance {
    extern "system" {
        fn CreateMutexW(lpMutexAttributes: *mut u8, bInitialOwner: i32, lpName: *const u16) -> isize;
        fn GetLastError() -> u32;
    }
    const ERROR_ALREADY_EXISTS: u32 = 183;

    pub fn ensure_single_instance() -> bool {
        let name: Vec<u16> = "Global\\FinderAppSpotlightSingleton\0".encode_utf16().collect();
        unsafe {
            let handle = CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr());
            if handle == 0 || GetLastError() == ERROR_ALREADY_EXISTS {
                crate::force_foreground();
                return false;
            }
        }
        true
    }
}

#[cfg(target_os = "windows")]
mod global_hotkey {
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, OnceLock, RwLock};
    use tauri::AppHandle;

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static IS_VISIBLE: OnceLock<Arc<AtomicBool>> = OnceLock::new();
    static CURRENT_HOTKEY: OnceLock<RwLock<HotkeyConfig>> = OnceLock::new();
    static mut HOOK: isize = 0;

    const WH_KEYBOARD_LL: i32 = 13;
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_KEYUP: u32 = 0x0101;
    const WM_SYSKEYDOWN: u32 = 0x0104;
    const WM_SYSKEYUP: u32 = 0x0105;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct HotkeyConfig {
        pub ctrl: bool,
        pub alt: bool,
        pub shift: bool,
        pub win: bool,
        pub vk: u32,
        pub is_single_modifier: bool,
    }

    impl HotkeyConfig {
        pub fn parse(s: &str) -> Self {
            let clean = s.trim();
            let lower = clean.to_lowercase().replace(' ', "");

            if lower == "rightalt" || lower == "ralt" || lower == "altright" {
                return Self {
                    ctrl: false,
                    alt: false,
                    shift: false,
                    win: false,
                    vk: 0xA5, // VK_RMENU
                    is_single_modifier: true,
                };
            }
            if lower == "leftalt" || lower == "lalt" || lower == "altleft" {
                return Self {
                    ctrl: false,
                    alt: false,
                    shift: false,
                    win: false,
                    vk: 0xA4, // VK_LMENU
                    is_single_modifier: true,
                };
            }
            if lower == "alt" {
                return Self {
                    ctrl: false,
                    alt: false,
                    shift: false,
                    win: false,
                    vk: 0xA5,
                    is_single_modifier: true,
                };
            }

            let parts: Vec<&str> = clean.split('+').map(|p| p.trim()).collect();
            let mut ctrl = false;
            let mut alt = false;
            let mut shift = false;
            let mut win = false;
            let mut vk = 0xA5;
            let mut has_key = false;

            for part in &parts {
                let p_lower = part.to_lowercase();
                match p_lower.as_str() {
                    "ctrl" | "control" => ctrl = true,
                    "alt" | "option" => alt = true,
                    "shift" => shift = true,
                    "super" | "win" | "cmd" | "meta" => win = true,
                    "space" => { vk = 0x20; has_key = true; },
                    "tab" => { vk = 0x09; has_key = true; },
                    "enter" | "return" => { vk = 0x0D; has_key = true; },
                    "esc" | "escape" => { vk = 0x1B; has_key = true; },
                    "tilde" | "`" | "~" => { vk = 0xC0; has_key = true; },
                    "f1" => { vk = 0x70; has_key = true; },
                    "f2" => { vk = 0x71; has_key = true; },
                    "f3" => { vk = 0x72; has_key = true; },
                    "f4" => { vk = 0x73; has_key = true; },
                    "f5" => { vk = 0x74; has_key = true; },
                    "f6" => { vk = 0x75; has_key = true; },
                    "f7" => { vk = 0x76; has_key = true; },
                    "f8" => { vk = 0x77; has_key = true; },
                    "f9" => { vk = 0x78; has_key = true; },
                    "f10" => { vk = 0x79; has_key = true; },
                    "f11" => { vk = 0x7A; has_key = true; },
                    "f12" => { vk = 0x7B; has_key = true; },
                    other => {
                        if other.len() == 1 {
                            let c = other.chars().next().unwrap().to_ascii_uppercase();
                            if c.is_ascii_alphanumeric() {
                                vk = c as u32;
                                has_key = true;
                            }
                        }
                    }
                }
            }

            if !has_key && (ctrl || alt || shift || win) {
                return Self {
                    ctrl: false,
                    alt: false,
                    shift: false,
                    win: false,
                    vk: 0xA5,
                    is_single_modifier: true,
                };
            }

            Self {
                ctrl,
                alt,
                shift,
                win,
                vk,
                is_single_modifier: false,
            }
        }
    }

    #[repr(C)]
    struct KBDLLHOOKSTRUCT {
        vk_code: u32,
        scan_code: u32,
        flags: u32,
        time: u32,
        extra_info: usize,
    }

    extern "system" {
        fn SetWindowsHookExW(
            id_hook: i32,
            lpfn: unsafe extern "system" fn(i32, usize, usize) -> isize,
            hmod: isize,
            thread_id: u32,
        ) -> isize;
        fn CallNextHookEx(hhk: isize, n_code: i32, w_param: usize, l_param: usize) -> isize;
        fn GetMessageW(msg: *mut u8, hwnd: isize, filter_min: u32, filter_max: u32) -> i32;
        fn GetAsyncKeyState(vKey: i32) -> i16;
    }

    unsafe extern "system" fn hook_proc(n_code: i32, w_param: usize, l_param: usize) -> isize {
        if n_code >= 0 {
            let msg = w_param as u32;
            let info = &*(l_param as *const KBDLLHOOKSTRUCT);

            if let Some(rw) = CURRENT_HOTKEY.get() {
                if let Ok(config) = rw.read() {
                    if config.is_single_modifier {
                        if (msg == WM_KEYUP || msg == WM_SYSKEYUP) && info.vk_code == config.vk {
                            if let (Some(app), Some(vis)) = (APP_HANDLE.get(), IS_VISIBLE.get()) {
                                crate::toggle_window_visibility(app, vis);
                            }
                        }
                    } else {
                        if (msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN) && info.vk_code == config.vk {
                            let ctrl_down = (GetAsyncKeyState(0x11) as u16 & 0x8000) != 0;
                            let alt_down = (GetAsyncKeyState(0x12) as u16 & 0x8000) != 0;
                            let shift_down = (GetAsyncKeyState(0x10) as u16 & 0x8000) != 0;
                            let win_down = ((GetAsyncKeyState(0x5B) as u16 & 0x8000) != 0) || ((GetAsyncKeyState(0x5C) as u16 & 0x8000) != 0);

                            if ctrl_down == config.ctrl && alt_down == config.alt && shift_down == config.shift && win_down == config.win {
                                if let (Some(app), Some(vis)) = (APP_HANDLE.get(), IS_VISIBLE.get()) {
                                    crate::toggle_window_visibility(app, vis);
                                }
                                return 1; // Consume key event
                            }
                        }
                    }
                }
            }
        }
        CallNextHookEx(HOOK, n_code, w_param, l_param)
    }

    pub fn set_hotkey(hotkey_str: &str) {
        let config = HotkeyConfig::parse(hotkey_str);
        if let Some(rw) = CURRENT_HOTKEY.get() {
            if let Ok(mut lock) = rw.write() {
                *lock = config;
            }
        } else {
            let _ = CURRENT_HOTKEY.set(RwLock::new(config));
        }
    }

    pub fn start(app: AppHandle, is_visible: Arc<AtomicBool>, initial_hotkey: &str) {
        let _ = APP_HANDLE.set(app);
        let _ = IS_VISIBLE.set(is_visible);
        set_hotkey(initial_hotkey);

        std::thread::spawn(move || unsafe {
            HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, hook_proc, 0, 0);
            let mut msg = [0u8; 48];
            while GetMessageW(msg.as_mut_ptr(), 0, 0, 0) > 0 {}
        });
    }
}

// ==========================================================================
// Application Entrypoint
// ==========================================================================

fn main() {
    #[cfg(target_os = "windows")]
    if !single_instance::ensure_single_instance() {
        std::process::exit(0);
    }

    let cached_apps = AppScanner::get_installed_apps();
    let is_visible = Arc::new(AtomicBool::new(true));
    let app_state = Arc::new(AppState {
        monitor: HardwareMonitor::new(),
        cached_apps,
        is_visible: Arc::clone(&is_visible),
    });

    let state_for_tray = Arc::clone(&app_state);
    let state_for_setup = Arc::clone(&app_state);
    let state_for_blur = Arc::clone(&app_state);

    tauri::Builder::default()
        .manage(Arc::clone(&app_state))
        .plugin(tauri_plugin_shell::init())
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if !should_ignore_blur() {
                    let _ = window.hide();
                    state_for_blur.is_visible.store(false, Ordering::SeqCst);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            search,
            launch_app,
            focus_window,
            execute_command,
            get_system_telemetry,
            get_weather,
            get_notes,
            save_note,
            delete_note,
            get_settings,
            save_settings,
            hide_window
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            #[cfg(target_os = "windows")]
            {
                let settings = StorageManager::load_settings();
                let initial_hotkey = settings.get("hotkey").and_then(|v| v.as_str()).unwrap_or("Right Alt");
                global_hotkey::start(app_handle.clone(), Arc::clone(&state_for_setup.is_visible), initial_hotkey);
            }

            // Setup Tray Menu
            if let Ok(toggle_item) = MenuItem::with_id(&app_handle, "toggle", "Toggle Finder", true, None::<&str>) {
                if let Ok(settings_item) = MenuItem::with_id(&app_handle, "settings", "Preferences...", true, None::<&str>) {
                    if let Ok(quit_item) = MenuItem::with_id(&app_handle, "quit", "Quit Finder", true, None::<&str>) {
                        if let Ok(tray_menu) = Menu::with_items(&app_handle, &[&toggle_item, &settings_item, &quit_item]) {
                            let state_clone = Arc::clone(&state_for_tray);
                            let app_handle_tray = app_handle.clone();

                            let mut tray_builder = TrayIconBuilder::new()
                                .menu(&tray_menu)
                                .show_menu_on_left_click(false)
                                .on_menu_event(move |_app, event| match event.id.as_ref() {
                                    "toggle" => {
                                        toggle_window_visibility(&app_handle_tray, &state_clone.is_visible);
                                    }
                                    "settings" => {
                                        if let Some(win) = app_handle_tray.get_webview_window("main") {
                                            let _ = win.show();
                                            let _ = win.set_focus();
                                            let _ = win.emit("open-settings", ());
                                        }
                                    }
                                    "quit" => {
                                        std::process::exit(0);
                                    }
                                    _ => {}
                                });

                            if let Some(icon) = app.default_window_icon() {
                                tray_builder = tray_builder.icon(icon.clone());
                            }

                            let _ = tray_builder.build(app);
                        }
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Finder application");
}
