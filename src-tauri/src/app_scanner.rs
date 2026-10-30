use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub title: String,
    pub description: String,
    pub path: String,
    pub category: String,
    pub icon: String,
}

pub struct AppScanner;

impl AppScanner {
    pub fn get_installed_apps() -> Vec<AppEntry> {
        let mut apps = Vec::new();

        // 1. Built-in Core System Apps & Utilities
        apps.extend(Self::get_system_utilities());

        // 2. OS-specific Scans
        #[cfg(target_os = "windows")]
        {
            apps.extend(Self::scan_windows_start_menu());
        }

        #[cfg(target_os = "linux")]
        {
            apps.extend(Self::scan_linux_desktop_files());
        }

        // Deduplicate by title
        apps.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        apps.dedup_by(|a, b| a.title.to_lowercase() == b.title.to_lowercase());

        apps
    }

    fn get_system_utilities() -> Vec<AppEntry> {
        #[cfg(target_os = "windows")]
        {
            vec![
                AppEntry { title: "Command Prompt".into(), description: "Windows Command Line".into(), path: "cmd.exe".into(), category: "Apps".into(), icon: "💻".into() },
                AppEntry { title: "PowerShell".into(), description: "Windows PowerShell".into(), path: "powershell.exe".into(), category: "Apps".into(), icon: "⚡".into() },
                AppEntry { title: "Windows Terminal".into(), description: "Modern Tabbed Terminal".into(), path: "wt.exe".into(), category: "Apps".into(), icon: "⚡".into() },
                AppEntry { title: "Task Manager".into(), description: "System Monitor & Processes".into(), path: "taskmgr.exe".into(), category: "Commands".into(), icon: "📊".into() },
                AppEntry { title: "File Explorer".into(), description: "Browse Files & Folders".into(), path: "explorer.exe".into(), category: "Apps".into(), icon: "📁".into() },
                AppEntry { title: "Settings".into(), description: "Windows Settings".into(), path: "ms-settings:".into(), category: "Apps".into(), icon: "⚙️".into() },
                AppEntry { title: "Control Panel".into(), description: "System Control Panel".into(), path: "control.exe".into(), category: "Apps".into(), icon: "🎛️".into() },
                AppEntry { title: "Snipping Tool".into(), description: "Take Screenshots".into(), path: "ms-screenclip:".into(), category: "Commands".into(), icon: "✂️".into() },
                AppEntry { title: "Calculator".into(), description: "Standard & Scientific Calculator".into(), path: "calc.exe".into(), category: "Apps".into(), icon: "🧮".into() },
                AppEntry { title: "Notepad".into(), description: "Text Editor".into(), path: "notepad.exe".into(), category: "Apps".into(), icon: "📝".into() },
                AppEntry { title: "Paint".into(), description: "Image Editor".into(), path: "mspaint.exe".into(), category: "Apps".into(), icon: "🎨".into() },
                AppEntry { title: "Registry Editor".into(), description: "Windows Registry".into(), path: "regedit.exe".into(), category: "Commands".into(), icon: "🛠️".into() },
                AppEntry { title: "Device Manager".into(), description: "Hardware & Drivers".into(), path: "devmgmt.msc".into(), category: "Commands".into(), icon: "🔌".into() },
                AppEntry { title: "Disk Management".into(), description: "Partitions & Disks".into(), path: "diskmgmt.msc".into(), category: "Commands".into(), icon: "💾".into() },
                AppEntry { title: "Services".into(), description: "Windows System Services".into(), path: "services.msc".into(), category: "Commands".into(), icon: "⚙️".into() },
                AppEntry { title: "Event Viewer".into(), description: "System Logs & Events".into(), path: "eventvwr.msc".into(), category: "Commands".into(), icon: "📋".into() },
                AppEntry { title: "System Information".into(), description: "Hardware & Software Specs".into(), path: "msinfo32.exe".into(), category: "Commands".into(), icon: "ℹ️".into() },
            ]
        }

        #[cfg(target_os = "linux")]
        {
            vec![
                AppEntry { title: "Terminal".into(), description: "Command Line Terminal".into(), path: "x-terminal-emulator".into(), category: "Apps".into(), icon: "💻".into() },
                AppEntry { title: "File Manager".into(), description: "Open Files".into(), path: "xdg-open .".into(), category: "Apps".into(), icon: "📁".into() },
                AppEntry { title: "System Settings".into(), description: "Desktop Environment Settings".into(), path: "gnome-control-center".into(), category: "Apps".into(), icon: "⚙️".into() },
                AppEntry { title: "System Monitor".into(), description: "View Processes & Hardware".into(), path: "gnome-system-monitor".into(), category: "Commands".into(), icon: "📊".into() },
                AppEntry { title: "Calculator".into(), description: "Calculator".into(), path: "gnome-calculator".into(), category: "Apps".into(), icon: "🧮".into() },
                AppEntry { title: "Text Editor".into(), description: "Default Editor".into(), path: "xdg-open".into(), category: "Apps".into(), icon: "📝".into() },
            ]
        }
    }

    #[cfg(target_os = "windows")]
    fn scan_windows_start_menu() -> Vec<AppEntry> {
        let mut apps = Vec::new();
        let mut search_paths = Vec::new();

        // Common Start Menu
        search_paths.push(PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"));

        // User Start Menu
        if let Ok(appdata) = std::env::var("APPDATA") {
            search_paths.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs"));
        }

        // User Local Programs
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            search_paths.push(PathBuf::from(local_appdata).join(r"Programs"));
        }

        for base_path in search_paths {
            if !base_path.exists() {
                continue;
            }

            for entry in walkdir::WalkDir::new(&base_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "lnk" || ext_lower == "exe" || ext_lower == "url" {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let stem_lower = file_stem.to_lowercase();
                            if stem_lower.contains("uninstall") || stem_lower.contains("remove") || stem_lower.starts_with("help") {
                                continue;
                            }

                            apps.push(AppEntry {
                                title: file_stem.to_string(),
                                description: path.to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                category: "Apps".into(),
                                icon: "🚀".into(),
                            });
                        }
                    }
                }
            }
        }

        apps
    }

    #[cfg(target_os = "linux")]
    fn scan_linux_desktop_files() -> Vec<AppEntry> {
        let mut apps = Vec::new();
        let mut paths = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
        ];

        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".local/share/applications"));
        }

        for base_path in paths {
            if !base_path.exists() {
                continue;
            }

            for entry in walkdir::WalkDir::new(base_path).max_depth(2).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let mut name = None;
                        let mut exec = None;
                        let mut comment = None;
                        let mut no_display = false;

                        for line in content.lines() {
                            let line = line.trim();
                            if line.starts_with("Name=") && name.is_none() {
                                name = Some(line["Name=".len()..].trim().to_string());
                            } else if line.starts_with("Exec=") && exec.is_none() {
                                let raw_exec = line["Exec=".len()..].trim();
                                let clean_exec = raw_exec.split_whitespace().next().unwrap_or(raw_exec).to_string();
                                exec = Some(clean_exec);
                            } else if line.starts_with("Comment=") && comment.is_none() {
                                comment = Some(line["Comment=".len()..].trim().to_string());
                            } else if line == "NoDisplay=true" {
                                no_display = true;
                                break;
                            }
                        }

                        if !no_display {
                            if let (Some(title), Some(cmd)) = (name, exec) {
                                apps.push(AppEntry {
                                    title,
                                    description: comment.unwrap_or_else(|| cmd.clone()),
                                    path: cmd,
                                    category: "Apps".into(),
                                    icon: "🚀".into(),
                                });
                            }
                        }
                    }
                }
            }
        }

        apps
    }
}
