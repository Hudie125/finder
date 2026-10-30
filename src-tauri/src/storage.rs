use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteItem {
    pub text: String,
    pub timestamp: String,
}

pub struct StorageManager;

impl StorageManager {
    fn get_storage_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        let dir = std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Finder"))
            .unwrap_or_else(|_| PathBuf::from(".finder"));

        #[cfg(not(target_os = "windows"))]
        let dir = std::env::var("HOME")
            .map(|p| PathBuf::from(p).join(".config").join("finder"))
            .unwrap_or_else(|_| PathBuf::from(".finder"));

        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    }

    pub fn load_notes() -> Vec<NoteItem> {
        let path = Self::get_storage_dir().join("search_notes.json");
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn save_notes(notes: &[NoteItem]) {
        let path = Self::get_storage_dir().join("search_notes.json");
        if let Ok(json) = serde_json::to_string_pretty(notes) {
            let _ = fs::write(path, json);
        }
    }

    pub fn load_settings() -> serde_json::Value {
        let path = Self::get_storage_dir().join("search_settings.json");
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({
                "hotkey": "Right Alt",
                "whisperEngine": "groq",
                "aiProvider": "pollinations",
                "theme": "liquid-dark"
            })
        }
    }

    pub fn save_settings(settings: &serde_json::Value) {
        let path = Self::get_storage_dir().join("search_settings.json");
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = fs::write(path, json);
        }
    }
}
