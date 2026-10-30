# ✦ Finder

<p align="center">
  <b>Next-Generation Spotlight Launcher & Voice Assistant</b><br>
  Built with <b>Rust</b>, <b>Tauri</b>, and modern Web Technologies (HTML/CSS/JS).<br>
  Designed in a pristine <b>Monochrome UI + Liquid Glass</b> aesthetic with smooth Apple-style spring animations.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/OS-Windows%2010%2F11%20%7C%20Arch%20Linux-black?style=for-the-badge&logo=windows&logoColor=white" />
  <img src="https://img.shields.io/badge/Backend-Rust%20%2B%20Tauri%20v2-black?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Voice-Whisper%20AI-black?style=for-the-badge&logo=openai&logoColor=white" />
  <img src="https://img.shields.io/badge/Style-Liquid%20Glass-black?style=for-the-badge&logo=apple&logoColor=white" />
</p>

---

## ✨ Features Overview

### 1. 🪞 Monochrome UI + Liquid Glass Aesthetics
* **Apple Fluid Animations**: Native spring easing (`cubic-bezier(0.16, 1, 0.3, 1)`), zero-lag responsive cards, and liquid light sheen that reacts to mouse movement.
* **Frosted Acrylic / Mica Blur**: High-density backdrop blur with subtle specular inner borders and obsidian shadow depth.
* **Instant Summon**: Toggled with a global hotkey (default: <kbd>Right Alt</kbd> or <kbd>Alt+Space</kbd>).

---

### 2. 🎙️ Whisper Voice Search & Smart Voice Commands
* **Whisper Integration**: High-accuracy speech recognition powered by Groq Whisper Cloud (Large-v3, <300ms latency), Local Whisper (`localhost:8000`), or built-in Web Speech API.
* **Real-time Liquid Wave Visualizer**: Animated soundwave canvas responds dynamically to voice amplitude.
* **Smart Voice Command Execution**:
  * *"Запусти калькулятор"* / *"Open terminal"* ➔ Automatically launches application.
  * *"Заблокируй компьютер"* / *"Lock screen"* ➔ Locks your workstation immediately.
  * *"Какая погода в Москве"* / *"Weather in Berlin"* ➔ Renders live weather forecast.
  * *"Таймер на 15 минут"* / *"Timer 25 minutes"* ➔ Sets background timer with desktop alert.
  * *"Спроси у нейросети..."* / *"Ask AI..."* ➔ Routes prompt directly to Cloud AI.

---

### 3. ✦ Free Cloud AI Intelligence
* **Zero-Setup AI Assistant**: Built-in free cloud neural network (Pollinations / DuckDuckGo text models) — **no API key required!**
* **Optional Gemini & Groq**: Plug in your free Google Gemini API key or Groq key for ultra-fast Llama 3.3 70B & Gemini 2.0 streaming.
* **Streaming Markdown Responses**: Formatted output with code syntax highlighting and one-click "Copy Code" buttons.
* **Trigger directly**: Type `? your question` in search or click the **AI Assistant** tab.

---

### 4. 🚀 Universal App Launcher & Window Switcher
* **Device Application Indexing**:
  * **Windows 10/11**: Start Menu programs, UWP/Store apps, PATH binaries, Windows tools (`cmd`, `wt`, `taskmgr`, `calc`, `regedit`, `services.msc`, `diskmgmt.msc`, etc.).
  * **Arch Linux**: `.desktop` entries in `/usr/share/applications` and `~/.local/share/applications`, plus `$PATH` binaries.
* **Run as Administrator**: Press <kbd>Ctrl</kbd> + <kbd>Enter</kbd> on any application to launch elevated (UAC / pkexec).
* **Active Window Switcher**: Live list of open GUI windows. Select any window and hit <kbd>Enter</kbd> to focus and bring it to front.

---

### 5. 🔍 Deep File & Content Search (Grep)
* Searches across Desktop, Documents, Downloads, Pictures, and Videos.
* **Content Search**: Type `content:term` or any query to grep inside code, markdown, txt, and json files with live snippet preview.

---

### 6. 🧮 Math Calculator & Currency Converter
* **Scientific Calculator**: Type expressions like `sqrt(144) * 2 + sin(pi/2)` or `45 * 18 - 12` for instant calculation and 1-click clipboard copy.
* **Live Currency Conversion**: Type `100 USD in RUB`, `50 EUR to USD`, `1000 JPY in EUR` for instant real-time rate calculation.

---

### 7. 📊 Real-time Hardware Telemetry & Weather
* **System Monitor**: Live CPU model name and percentage load gauge, RAM used/free/percentage, GPU activity gauge, and system uptime.
* **Live Weather Widget**: Auto-detects location via IP geolocation or custom city name, showing temperature, condition emoji, and forecast.

---

### 8. ⚡ Quick Commands & Tools
| Command | Action |
| :--- | :--- |
| `>lock` | Lock PC workstation session |
| `>sleep` | Put computer into suspend / sleep |
| `>reboot` | Reboot computer (10-second countdown) |
| `>shutdown` | Power off computer (10-second countdown) |
| `>cancel` | Abort pending shutdown / reboot |
| `>taskmgr` | Open Task Manager / System Monitor |
| `>settings` | Open Windows Settings / GNOME Control Center |
| `>snip` | Launch Snipping Tool / Screenshot |
| `>emptybin` | Permanently empty Recycle Bin / Trash |
| `>pw [length]` | Generate cryptographic password (16, 24, 32 chars) |
| `>rand` | Generate random number (1-100) |
| `>dice` | Roll 6-sided dice (1-6) |
| `>coin` | Flip coin (Heads / Tails) |
| `>timer 25m` | Start Pomodoro focus timer |
| `!note <text>` | Instantly save a quick note |
| `!notes` | Open Notes manager |

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Description |
| :--- | :--- |
| <kbd>Right Alt</kbd> or <kbd>Alt</kbd> + <kbd>Space</kbd> | Summon / Toggle Finder Window |
| <kbd>Ctrl</kbd> + <kbd>M</kbd> | Toggle Whisper Voice Recording |
| <kbd>Enter</kbd> | Open selected App / Execute Command / Focus Window |
| <kbd>Ctrl</kbd> + <kbd>Enter</kbd> | Launch selected application **As Administrator** |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> | Copy selected result / calculation to clipboard |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Open Finder Preferences & Settings Modal |
| <kbd>Esc</kbd> | Clear search query or hide Finder window |

---

## 📦 Automated Installation & Setup

### 🪟 Windows 10 & 11
Run the automated installer script in PowerShell. It automatically checks and installs any missing prerequisites (Rust, Node, Build Tools), compiles the app, and sets up Start Menu shortcuts:

```powershell
# Run from repository folder:
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

Or build manually:
```powershell
npm install
npm run build
```

---

### 🐧 Arch Linux
Run the automated bash installer for Arch Linux. It installs pacman dependencies (`rust`, `cargo`, `nodejs`, `webkit2gtk-4.1`, `libayatana-appindicator`), builds Finder, and installs it to `/usr/local/bin/finder` with `.desktop` menu integration:

```bash
chmod +x ./install.sh
./install.sh
```

Or build using native `PKGBUILD`:
```bash
makepkg -si
```

---

## 🚀 GitHub Actions Continuous Delivery

Every release tag pushed to GitHub (e.g. `v2.0.0`) automatically triggers `.github/workflows/release.yml`, building:
* **Windows**: Standalone `.exe` & `.msi` installers.
* **Linux**: Portable `.AppImage` & `.deb` packages.

### Publish to your GitHub Repository:
Run the helper script:
```powershell
# On Windows:
.\publish-repo.ps1

# On Linux:
./publish-repo.sh
```

---

## 📄 License
Released under the [MIT License](LICENSE).
