use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandItem {
    pub title: String,
    pub description: String,
    pub cmd: String,
    pub category: String,
    pub icon: String,
}

pub struct CommandRunner;

impl CommandRunner {
    pub fn get_all_commands() -> Vec<CommandItem> {
        vec![
            CommandItem { title: "Lock PC".into(), description: "Lock current user session".into(), cmd: "lock".into(), category: "Commands".into(), icon: "🔒".into() },
            CommandItem { title: "Sleep".into(), description: "Put computer to sleep / suspend".into(), cmd: "sleep".into(), category: "Commands".into(), icon: "🌙".into() },
            CommandItem { title: "Reboot (10s)".into(), description: "Restart system in 10 seconds".into(), cmd: "reboot".into(), category: "Commands".into(), icon: "🔄".into() },
            CommandItem { title: "Shutdown (10s)".into(), description: "Power off system in 10 seconds".into(), cmd: "shutdown".into(), category: "Commands".into(), icon: "🛑".into() },
            CommandItem { title: "Cancel Shutdown".into(), description: "Abort scheduled reboot or shutdown".into(), cmd: "cancel".into(), category: "Commands".into(), icon: "❌".into() },
            CommandItem { title: "Empty Recycle Bin".into(), description: "Permanently delete trash".into(), cmd: "emptybin".into(), category: "Commands".into(), icon: "🗑️".into() },
            CommandItem { title: "Task Manager".into(), description: "Inspect system performance and processes".into(), cmd: "taskmgr".into(), category: "Commands".into(), icon: "📊".into() },
            CommandItem { title: "System Settings".into(), description: "Open OS settings app".into(), cmd: "settings".into(), category: "Commands".into(), icon: "⚙️".into() },
            CommandItem { title: "Snipping Tool".into(), description: "Capture screen clip or screenshot".into(), cmd: "snip".into(), category: "Commands".into(), icon: "✂️".into() },
            CommandItem { title: "Terminal".into(), description: "Open Command Terminal".into(), cmd: "terminal".into(), category: "Commands".into(), icon: "💻".into() },
            CommandItem { title: "PowerShell".into(), description: "Open PowerShell".into(), cmd: "ps".into(), category: "Commands".into(), icon: "⚡".into() },
            CommandItem { title: "Registry Editor".into(), description: "Windows Registry".into(), cmd: "regedit".into(), category: "Commands".into(), icon: "🛠️".into() },
            CommandItem { title: "Password 16".into(), description: "Generate secure 16-character password".into(), cmd: "pw 16".into(), category: "Commands".into(), icon: "🔑".into() },
            CommandItem { title: "Password 24".into(), description: "Generate strong 24-character password".into(), cmd: "pw 24".into(), category: "Commands".into(), icon: "🔐".into() },
            CommandItem { title: "Password 32".into(), description: "Generate ultra-secure 32-character password".into(), cmd: "pw 32".into(), category: "Commands".into(), icon: "🛡️".into() },
            CommandItem { title: "Random (1-100)".into(), description: "Generate random number between 1 and 100".into(), cmd: "rand".into(), category: "Commands".into(), icon: "🎲".into() },
            CommandItem { title: "Dice Roll".into(), description: "Roll a 6-sided dice (1-6)".into(), cmd: "dice".into(), category: "Commands".into(), icon: "🎲".into() },
            CommandItem { title: "Coin Flip".into(), description: "Flip a coin: Heads or Tails".into(), cmd: "coin".into(), category: "Commands".into(), icon: "🪙".into() },
            CommandItem { title: "Timer 25m (Pomodoro)".into(), description: "Start 25-minute focus session".into(), cmd: "timer 25m".into(), category: "Commands".into(), icon: "⏱️".into() },
            CommandItem { title: "Timer 5m".into(), description: "Start 5-minute break timer".into(), cmd: "timer 5m".into(), category: "Commands".into(), icon: "⏱️".into() },
            CommandItem { title: "Timer 15m".into(), description: "Start 15-minute timer".into(), cmd: "timer 15m".into(), category: "Commands".into(), icon: "⏱️".into() },
            CommandItem { title: "Timer 1h".into(), description: "Start 1-hour timer".into(), cmd: "timer 1h".into(), category: "Commands".into(), icon: "⏱️".into() },
        ]
    }

    pub fn execute(command_str: &str) -> Result<String, String> {
        let cmd = command_str.trim().to_lowercase();
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let action = parts.first().copied().unwrap_or("");

        match action {
            "lock" => {
                #[cfg(target_os = "windows")]
                {
                    extern "system" {
                        fn LockWorkStation() -> i32;
                    }
                    unsafe { let _ = LockWorkStation(); }
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("loginctl").arg("lock-session").spawn();
                }
                Ok("Session locked".into())
            }
            "sleep" => {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("rundll32.exe")
                        .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("systemctl").arg("suspend").spawn();
                }
                Ok("System going to sleep".into())
            }
            "reboot" => {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("shutdown").args(["/r", "/t", "10"]).spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("systemctl").arg("reboot").spawn();
                }
                Ok("Reboot scheduled in 10s".into())
            }
            "shutdown" => {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("shutdown").args(["/s", "/t", "10"]).spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
                }
                Ok("Shutdown scheduled in 10s".into())
            }
            "cancel" => {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("shutdown").arg("/a").spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("shutdown").arg("-c").spawn();
                }
                Ok("Shutdown canceled".into())
            }
            "emptybin" => {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("powershell")
                        .args(["-NoProfile", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"])
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("rm").args(["-rf", "~/.local/share/Trash/*"]).spawn();
                }
                Ok("Recycle bin emptied".into())
            }
            "taskmgr" => {
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("taskmgr.exe").spawn(); }
                #[cfg(target_os = "linux")]
                { let _ = std::process::Command::new("gnome-system-monitor").spawn(); }
                Ok("Task Manager opened".into())
            }
            "settings" => {
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("explorer.exe").arg("ms-settings:").spawn(); }
                #[cfg(target_os = "linux")]
                { let _ = std::process::Command::new("gnome-control-center").spawn(); }
                Ok("Settings opened".into())
            }
            "snip" => {
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("explorer.exe").arg("ms-screenclip:").spawn(); }
                #[cfg(target_os = "linux")]
                { let _ = std::process::Command::new("spectacle").spawn(); }
                Ok("Screen snip launched".into())
            }
            "terminal" | "wt" => {
                #[cfg(target_os = "windows")]
                {
                    if std::process::Command::new("wt.exe").spawn().is_err() {
                        let _ = std::process::Command::new("powershell.exe").spawn();
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("x-terminal-emulator").spawn();
                }
                Ok("Terminal launched".into())
            }
            "ps" => {
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("powershell.exe").spawn(); }
                #[cfg(target_os = "linux")]
                { let _ = std::process::Command::new("pwsh").spawn(); }
                Ok("PowerShell launched".into())
            }
            "regedit" => {
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("regedit.exe").spawn(); }
                Ok("Registry Editor launched".into())
            }
            "pw" => {
                let len: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(24);
                let pw = Self::generate_password(len);
                Ok(format!("Generated Password: {}", pw))
            }
            "rand" => {
                let mut rng = rand::thread_rng();
                let num: i32 = rng.gen_range(1..=100);
                Ok(format!("Random Number (1-100): {}", num))
            }
            "dice" => {
                let mut rng = rand::thread_rng();
                let dice: i32 = rng.gen_range(1..=6);
                Ok(format!("Dice Roll: 🎲 {}", dice))
            }
            "coin" => {
                let mut rng = rand::thread_rng();
                let is_heads: bool = rng.gen();
                let result = if is_heads { "Heads 🪙" } else { "Tails 🪙" };
                Ok(format!("Coin Flip: {}", result))
            }
            "timer" => {
                let duration_str = parts.get(1).copied().unwrap_or("5m");
                let secs = Self::parse_duration_seconds(duration_str);
                let dur_owned = duration_str.to_string();
                
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
                    #[cfg(target_os = "windows")]
                    {
                        let msg = format!("Timer Finished: {}", dur_owned);
                        let _ = std::process::Command::new("powershell")
                            .args(["-NoProfile", "-Command", &format!("[System.Windows.Forms.MessageBox]::Show('{}', 'Finder Timer')", msg)])
                            .spawn();
                    }
                });

                Ok(format!("Timer set for {}", duration_str))
            }
            _ => Err(format!("Unknown command: {}", action)),
        }
    }

    fn generate_password(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789\
                                !@#$%^&*()-_=+[]{}";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn parse_duration_seconds(s: &str) -> u64 {
        let clean = s.trim().to_lowercase();
        if let Some(num_str) = clean.strip_suffix('s') {
            num_str.parse().unwrap_or(30)
        } else if let Some(num_str) = clean.strip_suffix('m') {
            num_str.parse::<u64>().unwrap_or(5) * 60
        } else if let Some(num_str) = clean.strip_suffix('h') {
            num_str.parse::<u64>().unwrap_or(1) * 3600
        } else {
            clean.parse().unwrap_or(5) * 60
        }
    }
}
