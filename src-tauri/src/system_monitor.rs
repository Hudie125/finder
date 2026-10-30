use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetry {
    pub cpu_name: String,
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub ram_usage_percent: f64,
    pub gpu_name: String,
    pub gpu_usage_percent: f32,
    pub os_name: String,
    pub uptime: String,
}

pub struct HardwareMonitor {
    sys: Mutex<System>,
}

impl HardwareMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys: Mutex::new(sys),
        }
    }

    pub fn get_telemetry(&self) -> SystemTelemetry {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let cpu_usage = sys.global_cpu_usage();
        let cpu_cores = sys.cpus().len();
        let cpu_name = sys.cpus().first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let used_mem_bytes = sys.used_memory();
        let total_mem_bytes = sys.total_memory();

        let ram_used_gb = (used_mem_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        let ram_total_gb = (total_mem_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        let ram_usage_percent = if total_mem_bytes > 0 {
            (used_mem_bytes as f64 / total_mem_bytes as f64) * 100.0
        } else {
            0.0
        };

        let uptime_secs = System::uptime();
        let hours = uptime_secs / 3600;
        let mins = (uptime_secs % 3600) / 60;
        let uptime = format!("{}h {}m", hours, mins);

        let os_name = System::long_os_version().unwrap_or_else(|| "Windows / Linux".to_string());
        let gpu_name = Self::detect_gpu_name();

        SystemTelemetry {
            cpu_name,
            cpu_usage,
            cpu_cores,
            ram_used_gb,
            ram_total_gb,
            ram_usage_percent,
            gpu_name,
            gpu_usage_percent: 15.0, // baseline estimate
            os_name,
            uptime,
        }
    }

    fn detect_gpu_name() -> String {
        #[cfg(target_os = "windows")]
        {
            // Try reading Display adapter from registry or wmic fallback
            let output = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "(Get-CimInstance Win32_VideoController | Select-Object -First 1).Name"])
                .output();

            if let Ok(out) = output {
                let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !name.is_empty() {
                    return name;
                }
            }
            "NVIDIA / AMD / Intel Graphics".to_string()
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(out) = std::process::Command::new("lspci").output() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    if line.contains("VGA compatible controller") || line.contains("3D controller") {
                        if let Some(pos) = line.find(": ") {
                            return line[pos + 2..].trim().to_string();
                        }
                    }
                }
            }
            "Linux GPU".to_string()
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            "Generic Graphics".to_string()
        }
    }
}
