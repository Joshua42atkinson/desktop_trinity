#!/bin/bash
# Antigravity Git Sync Script
# Auto-updates the repository with a clean state.

REPO_DIR="/home/joshua/antigravity"
cd "$REPO_DIR" || exit

echo "Starting Antigravity Git Sync..."

# Check if git is initialized
if [ ! -d ".git" ]; then
    echo "Initializing new git repository..."
    git init
    git checkout -b main
    # Attempt to add remote if it was known, but for now we just local sync
fi

# Add changes
git add .

# Check if there are changes to commit
if git diff-index --quiet HEAD --; then
    echo "No changes to sync."
else
    TIMESTAMP=$(date +"%Y-%m-%d %H:%M")
    git commit -m "Antigravity Session Sync: $TIMESTAMP"
    echo "Committed changes at $TIMESTAMP"
fi

# If a remote exists, try to push
REMOTE_URL=$(git remote get-url origin 2>/dev/null)
if [ -n "$REMOTE_URL" ]; then
    echo "Pushing to remote origin..."
    git push -u origin main
fi

echo "Sync complete."
