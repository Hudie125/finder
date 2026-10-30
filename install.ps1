# ==============================================================================
# Finder - Automated Windows 10/11 Installer & Environment Setup
# Installs all prerequisites, builds, and sets up Finder Spotlight Launcher
# ==============================================================================

[CmdletBinding()]
param (
    [switch]$SkipBuild,
    [switch]$LaunchAfterInstall
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "     Finder v2.0 - Windows 10/11 Automated Setup          " -ForegroundColor Cyan
Write-Host "     Monochrome Liquid Glass Spotlight Launcher           " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Check & Install Rust Toolchain
Write-Host "[1/5] Checking Rust & Cargo toolchain..." -ForegroundColor Yellow
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Cargo not found in PATH. Checking Rustup..." -ForegroundColor Yellow
    if (-not (Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe")) {
        Write-Host "Installing Rust via winget..." -ForegroundColor Green
        try {
            winget install --id Rustlang.Rustup -e --silent --accept-source-agreements --accept-package-agreements
        } catch {
            Write-Host "Winget failed, downloading rustup-init.exe directly..." -ForegroundColor Yellow
            $rustupInit = "$env:TEMP\rustup-init.exe"
            Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit
            Start-Process -FilePath $rustupInit -ArgumentList "-y", "--default-toolchain", "stable" -Wait -NoNewWindow
        }
    }
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "  Rust is ready: $(cargo --version)" -ForegroundColor Green
} else {
    Write-Host "  Rust installed to $env:USERPROFILE\.cargo\bin (will be active after reload)" -ForegroundColor Green
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

# 2. Check & Install Node.js & npm
Write-Host "`n[2/5] Checking Node.js environment..." -ForegroundColor Yellow
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "Node.js not found. Installing via winget..." -ForegroundColor Green
    winget install --id OpenJS.NodeJS.LTS -e --silent --accept-source-agreements --accept-package-agreements
    $env:PATH += ";$env:ProgramFiles\nodejs"
}
Write-Host "  Node is ready: $(node --version)" -ForegroundColor Green

# 3. Install NPM Dependencies & Tauri CLI
Write-Host "`n[3/5] Installing package dependencies..." -ForegroundColor Yellow
Set-Location -Path $PSScriptRoot
npm install --no-audit --prefer-offline

# 4. Build Release Binary
if (-not $SkipBuild) {
    Write-Host "`n[4/5] Building Finder in Release mode (Tauri + Rust)..." -ForegroundColor Yellow
    npm run build
}

# 5. Create Desktop & Start Menu Shortcuts
Write-Host "`n[5/5] Finalizing Installation & Shortcuts..." -ForegroundColor Yellow
$installDir = "$env:LOCALAPPDATA\Finder"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$releaseExe = "$PSScriptRoot\src-tauri\target\release\finder.exe"
$targetExe = "$installDir\Finder.exe"

if (Test-Path $releaseExe) {
    Copy-Item -Path $releaseExe -Destination $targetExe -Force
    Write-Host "  Copied binary to: $targetExe" -ForegroundColor Green
} else {
    Write-Host "  Note: Run 'npm run build' to produce the standalone binary." -ForegroundColor Yellow
}

# Create Start Menu shortcut
$WshShell = New-Object -ComObject WScript.Shell
$startMenuPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Finder.lnk"
$shortcut = $WshShell.CreateShortcut($startMenuPath)
$shortcut.TargetPath = if (Test-Path $targetExe) { $targetExe } else { "powershell.exe" }
$shortcut.WorkingDirectory = $installDir
$shortcut.Description = "Finder - Monochrome Liquid Glass Spotlight Launcher"
$shortcut.Save()

Write-Host "  Start Menu shortcut created: $startMenuPath" -ForegroundColor Green
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "   Finder has been successfully installed and configured!  " -ForegroundColor Green
Write-Host "   Summon shortcut: Right Alt (or Alt+Space)               " -ForegroundColor Green
Write-Host "   Voice Whisper:   Ctrl+M                                " -ForegroundColor Green
Write-Host "   AI Assistant:    Type '?' in search bar                " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green

if ($LaunchAfterInstall -and (Test-Path $targetExe)) {
    Write-Host "Launching Finder..." -ForegroundColor Cyan
    Start-Process -FilePath $targetExe
}
