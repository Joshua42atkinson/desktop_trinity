#!/bin/bash
# Setup Trinity for Terminal-Free Operation
# Run this once to install systemd service and desktop launcher

set -e

echo "🔮 Setting up Trinity for Always-On Operation..."

# 1. Build release binaries
echo "📦 Building release binaries..."
cargo build --release -p trinity-brain -p trinity-body

# 2. Install systemd service
echo "🔧 Installing systemd service..."
sudo cp /home/joshua/antigravity/trinity-genesis/trinity-brain.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable trinity-brain.service

# 3. Install desktop launcher
echo "🖥️ Installing desktop launcher..."
cp /home/joshua/antigravity/trinity-genesis/trinity-body.desktop ~/.local/share/applications/

# 4. Create placeholder icon if missing
if [ ! -f /home/joshua/antigravity/trinity-genesis/assets/trinity-icon.png ]; then
    echo "⚠️ No icon found, creating placeholder..."
    mkdir -p /home/joshua/antigravity/trinity-genesis/assets
    # Create a simple placeholder (just a colored square)
    convert -size 256x256 xc:#8B5CF6 /home/joshua/antigravity/trinity-genesis/assets/trinity-icon.png 2>/dev/null || \
    echo "   (Install imagemagick to auto-generate icon)"
fi

echo ""
echo "✅ Setup Complete!"
echo ""
echo "To start Trinity Brain (runs in background):"
echo "  sudo systemctl start trinity-brain"
echo ""
echo "To open Trinity UI:"
echo "  - Search for 'Trinity AI' in your app launcher"
echo "  - Or run: trinity-body"
echo ""
echo "To check Brain status:"
echo "  sudo systemctl status trinity-brain"
echo "  journalctl -u trinity-brain -f"
