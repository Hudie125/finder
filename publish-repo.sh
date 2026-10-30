#!/usr/bin/env bash
REPO_NAME="finder"
DESC="Monochrome Liquid Glass Spotlight Launcher with Voice (Whisper) & Free Cloud AI for Windows 10/11 & Arch Linux"

if command -v gh &> /dev/null; then
    if gh auth status &> /dev/null; then
        echo "Creating public GitHub repo..."
        gh repo create "$REPO_NAME" --public --description "$DESC" --source . --remote origin --push
        echo "Pushed to GitHub!"
    else
        echo "Please login to GitHub CLI:"
        gh auth login
        gh repo create "$REPO_NAME" --public --description "$DESC" --source . --remote origin --push
    fi
else
    echo "GitHub CLI not found. Please create repo manually and run:"
    echo "git remote add origin git@github.com:YOUR_USER/$REPO_NAME.git"
    echo "git push -u origin main"
fi
