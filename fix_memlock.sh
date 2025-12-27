#!/bin/bash
set -e

LIMITS_FILE="/etc/security/limits.conf"
BACKUP_FILE="${LIMITS_FILE}.bak.$(date +%s)"

echo ">>> Backing up $LIMITS_FILE to $BACKUP_FILE..."
sudo cp "$LIMITS_FILE" "$BACKUP_FILE"

echo ">>> Applying Unlimited Memlock configuration..."

# Check if already present to avoid duplication
if grep -q "soft memlock unlimited" "$LIMITS_FILE"; then
    echo "Configuration already exists."
else
    # Append the configuration
    echo "" | sudo tee -a "$LIMITS_FILE"
    echo "# Strix Halo Optimization: Allow unlimited memory locking for pinning large models" | sudo tee -a "$LIMITS_FILE"
    echo "* soft memlock unlimited" | sudo tee -a "$LIMITS_FILE"
    echo "* hard memlock unlimited" | sudo tee -a "$LIMITS_FILE"
    echo ">>> Configuration applied successfully."
fi

echo ">>> NOTE: You must REBOOT or Relogin for these changes to take effect."
