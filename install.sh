#!/usr/bin/env bash
# ==============================================================================
# Finder - Automated Arch Linux Installer & Setup Script
# Installs all pacman dependencies, builds, and installs Finder Spotlight
# ==============================================================================

set -e

echo -e "\033[1;36m==========================================================\033[0m"
echo -e "\033[1;36m     Finder v2.0 - Arch Linux Automated Setup             \033[0m"
echo -e "\033[1;36m     Monochrome Liquid Glass Spotlight Launcher           \033[0m"
echo -e "\033[1;36m==========================================================\033[0m"

# 1. Install System Dependencies via Pacman
echo -e "\n\033[1;33m[1/5] Installing dependencies via pacman...\033[0m"
if command -v pacman &> /dev/null; then
    sudo pacman -S --needed --noconfirm \
        rust \
        cargo \
        nodejs \
        npm \
        base-devel \
        webkit2gtk-4.1 \
        libayatana-appindicator \
        openssl \
        wmctrl \
        curl
else
    echo -e "\033[1;31mPacman not detected. Please install Rust, WebKitGTK-4.1, Node.js manually.\033[0m"
fi

# 2. Install NPM dependencies
echo -e "\n\033[1;33m[2/5] Installing package dependencies...\033[0m"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
npm install --no-audit

# 3. Build Tauri Release Binary
echo -e "\n\033[1;33m[3/5] Building Finder in Release mode (Tauri + Rust)...\033[0m"
npm run build

# 4. Install Binary & Desktop Entry
echo -e "\n\033[1;33m[4/5] Installing to system (/usr/local/bin)...\033[0m"
if [ -f "$SCRIPT_DIR/src-tauri/target/release/finder" ]; then
    sudo install -Dm755 "$SCRIPT_DIR/src-tauri/target/release/finder" /usr/local/bin/finder
    echo -e "\033[1;32m  Installed /usr/local/bin/finder\033[0m"
fi

# Install Icon
if [ -f "$SCRIPT_DIR/src-tauri/icons/icon.png" ]; then
    sudo install -Dm644 "$SCRIPT_DIR/src-tauri/icons/icon.png" /usr/share/icons/hicolor/512x512/apps/finder.png
fi

# Install .desktop entry
cat << 'EOF' | sudo tee /usr/share/applications/finder.desktop > /dev/null
[Desktop Entry]
Name=Finder
Comment=Monochrome Liquid Glass Spotlight Launcher with Voice & AI
Exec=/usr/local/bin/finder
Icon=finder
Terminal=false
Type=Application
Categories=Utility;System;
Keywords=spotlight;launcher;search;whisper;ai;
StartupNotify=false
EOF

# 5. Done
echo -e "\n\033[1;32m==========================================================\033[0m"
echo -e "\033[1;32m   Finder successfully installed on Arch Linux!           \033[0m"
echo -e "\033[1;32m   Run 'finder' from terminal or application launcher.    \033[0m"
echo -e "\033[1;32m   Summon shortcut: Right Alt (or Alt+Space)              \033[0m"
echo -e "\033[1;32m==========================================================\033[0m"
