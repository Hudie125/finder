use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub title: String,
    pub description: String,
    pub path: String,
    pub category: String,
    pub icon: String,
    pub snippet: Option<String>,
}

pub struct FileSearchEngine;

impl FileSearchEngine {
    pub fn search_files(query: &str, max_results: usize) -> Vec<FileSearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        let is_content_search = query_lower.starts_with("content:") || query_lower.starts_with("grep:");
        let search_term = if is_content_search {
            query_lower.split_once(':').map(|x| x.1.trim()).unwrap_or(&query_lower)
        } else {
            &query_lower
        };

        if search_term.is_empty() {
            return results;
        }

        let search_dirs = Self::get_user_search_directories();

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(dir)
                .max_depth(4)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if results.len() >= max_results {
                    break;
                }

                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let file_name_lower = file_name.to_lowercase();

                    if !is_content_search && file_name_lower.contains(search_term) {
                        results.push(FileSearchResult {
                            title: file_name.to_string(),
                            description: path.to_string_lossy().to_string(),
                            path: path.to_string_lossy().to_string(),
                            category: "Files".into(),
                            icon: Self::get_icon_for_file(path),
                            snippet: None,
                        });
                    } else if (is_content_search || search_term.len() >= 4) && Self::is_text_file(path) {
                        // Grep inside file
                        if let Some(snippet) = Self::grep_file_content(path, search_term) {
                            results.push(FileSearchResult {
                                title: file_name.to_string(),
                                description: path.to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                category: "Files".into(),
                                icon: "📄".into(),
                                snippet: Some(snippet),
                            });
                        }
                    }
                }
            }

            if results.len() >= max_results {
                break;
            }
        }

        results
    }

    fn get_user_search_directories() -> Vec<PathBuf> {
        let mut dirs_list = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                let base = PathBuf::from(userprofile);
                dirs_list.push(base.join("Desktop"));
                dirs_list.push(base.join("Documents"));
                dirs_list.push(base.join("Downloads"));
                dirs_list.push(base.join("Pictures"));
                dirs_list.push(base.join("Videos"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let base = PathBuf::from(home);
                dirs_list.push(base.join("Desktop"));
                dirs_list.push(base.join("Documents"));
                dirs_list.push(base.join("Downloads"));
                dirs_list.push(base.join("Pictures"));
                dirs_list.push(base.join("Videos"));
            }
        }

        dirs_list
    }

    fn is_text_file(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "txt" | "md" | "rs" | "py" | "js" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "csv" | "log" | "ini" | "sh" | "bat" | "ps1"
            )
        } else {
            false
        }
    }

    fn grep_file_content(path: &Path, term: &str) -> Option<String> {
        let metadata = std::fs::metadata(path).ok()?;
        // Skip files larger than 1.5MB to maintain lightning speed
        if metadata.len() > 1_500_000 {
            return None;
        }

        let content = std::fs::read_to_string(path).ok()?;
        let term_lower = term.to_lowercase();

        for line in content.lines() {
            if line.to_lowercase().contains(&term_lower) {
                let snippet = line.trim();
                let truncated: String = snippet.chars().take(120).collect();
                return Some(truncated);
            }
        }

        None
    }

    fn get_icon_for_file(path: &Path) -> String {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext.to_lowercase().as_str() {
                "pdf" => "📕",
                "doc" | "docx" | "odt" => "📘",
                "xls" | "xlsx" | "csv" => "📊",
                "ppt" | "pptx" => "📙",
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "🖼️",
                "mp3" | "wav" | "flac" | "aac" => "🎵",
                "mp4" | "mkv" | "avi" | "mov" => "🎬",
                "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
                "rs" | "py" | "js" | "ts" | "c" | "cpp" | "html" | "css" => "💻",
                _ => "📄",
            }
            .into()
        } else {
            "📄".into()
        }
    }
}
