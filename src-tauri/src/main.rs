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
    is_visible: AtomicBool,
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
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}

fn toggle_window_visibility(app: &AppHandle, is_visible: &AtomicBool) {
    if let Some(win) = app.get_webview_window("main") {
        let visible = is_visible.load(Ordering::SeqCst);
        if visible {
            let _ = win.hide();
            is_visible.store(false, Ordering::SeqCst);
        } else {
            let _ = win.show();
            let _ = win.set_focus();
            is_visible.store(true, Ordering::SeqCst);
            let _ = win.emit("focus-search", ());
        }
    }
}

// ==========================================================================
// Application Entrypoint
// ==========================================================================

fn main() {
    let cached_apps = AppScanner::get_installed_apps();
    let app_state = Arc::new(AppState {
        monitor: HardwareMonitor::new(),
        cached_apps,
        is_visible: AtomicBool::new(true),
    });

    let state_for_tray = Arc::clone(&app_state);

    tauri::Builder::default()
        .manage(Arc::clone(&app_state))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
