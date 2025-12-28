#!/bin/bash
# Trinity Auto-Sync Script
# Automatically commits and pushes changes to GitHub

# Exit on error
set -e

# Get current branch
BRANCH=$(git branch --show-current)
TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")

echo "🤖 Trinity Auto-Sync Initiated at $TIMESTAMP..."

# Check for custom SSH key
if [ -f "/home/joshua/antigravity/trinity_key" ]; then
    echo "🔑 Using Trinity SSH Key..."
    export GIT_SSH_COMMAND="ssh -i /home/joshua/antigravity/trinity_key -o IdentitiesOnly=yes"
fi

# Add all changes
git add .

# Commit if there are changes
if ! git diff-index --quiet HEAD --; then
    git commit -m "Trinity Auto-Sync: $TIMESTAMP"
    echo "✅ Committed changes."
else
    echo "ℹ️ No changes to commit."
fi

# Push
echo "🚀 Pushing to origin/$BRANCH..."
git push origin "$BRANCH"

echo "✨ Sync Complete!"
