#!/bin/bash
# Trinity Laptop Setup Script
# Run this on your laptop to connect to your desktop

echo "🔺 Trinity Laptop Setup"
echo "========================"

# Check if Tailscale is installed
if ! command -v tailscale &> /dev/null; then
    echo "📦 Installing Tailscale..."
    curl -fsSL https://tailscale.com/install.sh | sh
fi

# Start Tailscale and login
echo "🔑 Logging into Tailscale (use the same account as your desktop)..."
sudo tailscale up

# Test connection to desktop
echo ""
echo "🔗 Testing connection to Trinity Desktop..."
DESKTOP_IP="100.115.247.4"

if ping -c 1 $DESKTOP_IP &> /dev/null; then
    echo "✅ SUCCESS! Connected to Trinity Desktop at $DESKTOP_IP"
    echo ""
    echo "You can now:"
    echo "  • SSH:     ssh joshua@$DESKTOP_IP"
    echo "  • Trinity: http://$DESKTOP_IP:3001 (when backend is running)"
    echo ""
else
    echo "❌ Cannot reach desktop yet. Make sure:"
    echo "  1. Your desktop is running"
    echo "  2. You logged into Tailscale with the same Google account"
fi
