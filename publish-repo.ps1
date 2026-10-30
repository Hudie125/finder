# ==============================================================================
# Helper Script: Create Public GitHub Repository & Push Sources
# ==============================================================================

[CmdletBinding()]
param (
    [string]$RepoName = "finder",
    [string]$Description = "Monochrome Liquid Glass Spotlight Launcher with Voice (Whisper) & Free Cloud AI for Windows 10/11 & Arch Linux"
)

Write-Host "Checking GitHub CLI status..." -ForegroundColor Cyan

if (Get-Command gh -ErrorAction SilentlyContinue) {
    $authStatus = gh auth status 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Creating public GitHub repository: $RepoName..." -ForegroundColor Green
        gh repo create $RepoName --public --description "$Description" --source . --remote origin --push
        Write-Host "Repository created and pushed successfully to GitHub!" -ForegroundColor Green
        gh browse
        exit 0
    } else {
        Write-Host "You are not logged into GitHub CLI yet." -ForegroundColor Yellow
        Write-Host "Running: gh auth login..." -ForegroundColor Yellow
        gh auth login
        gh repo create $RepoName --public --description "$Description" --source . --remote origin --push
        exit 0
    }
} else {
    Write-Host "GitHub CLI (gh) not found. Manual git push steps:" -ForegroundColor Yellow
    Write-Host "1. Create an empty public repository on https://github.com/new"
    Write-Host "2. Run: git remote add origin https://github.com/<YOUR_USERNAME>/$RepoName.git"
    Write-Host "3. Run: git push -u origin main"
}
