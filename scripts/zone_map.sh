#!/bin/bash
# Zone Map - Visual overview of Trinity's zone structure
# Usage: ./scripts/zone_map.sh

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║                        TRINITY ZONE MAP                                   ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "                              ┌─────────────────┐"
echo "                              │   CONTEXT.md    │ ← Start here"
echo "                              │  (Master Doc)   │"
echo "                              └────────┬────────┘"
echo "                                       │"
echo "       ┌───────────────────────────────┼───────────────────────────────┐"
echo "       │                               │                               │"
echo "       ▼                               ▼                               ▼"
echo "┌─────────────┐                ┌─────────────┐                ┌─────────────┐"
echo "│  🧠 BRAIN   │                │  🎮 BODY    │                │ 🚂 IRON RD  │"
echo "│  /brain-dev │                │  /body-dev  │                │/iron-road   │"
echo "├─────────────┤                ├─────────────┤                ├─────────────┤"
echo "│ kernel      │◄──────────────►│ body        │                │ physics     │"
echo "│ brain       │  RPC/tarpc     │ client(wasm)│                │             │"
echo "│ protocol    │                │             │                │             │"
echo "└─────────────┘                └─────────────┘                └─────────────┘"
echo "       │"
echo "       │ WASM sandbox"
echo "       ▼"
echo "┌─────────────┐                ┌─────────────┐"
echo "│  🔧 TOOLS   │                │  📚 PETE    │"
echo "│  /tools-dev │                │  /pete-dev  │"
echo "├─────────────┤                ├─────────────┤"
echo "│ calculator  │                │ (migrating) │"
echo "│ code_editor │                │             │"
echo "└─────────────┘                └─────────────┘"
echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""

# Check build status of each zone
cd /home/joshua/antigravity/trinity-genesis

echo "ZONE STATUS:"
echo ""

# Brain zone
echo -n "🧠 BRAIN:     "
if cargo check -p trinity-kernel -p trinity-brain -p trinity-protocol --quiet 2>/dev/null; then
    echo "✅ Builds"
else
    echo "❌ Build errors"
fi

# Body zone
echo -n "🎮 BODY:      "
if cargo check -p trinity-body --quiet 2>/dev/null; then
    echo "✅ Builds"
else
    echo "❌ Build errors"
fi

# Iron Road zone
echo -n "🚂 IRON ROAD: "
if cargo check -p iron-road-physics --quiet 2>/dev/null; then
    echo "✅ Builds"
else
    echo "❌ Build errors"
fi

# Tools zone
echo -n "🔧 TOOLS:     "
if [ -f "plugins/calculator.wasm" ]; then
    echo "✅ WASM present"
else
    echo "⚠️  WASM not built"
fi

echo ""
echo "Run '/checkpoint' to verify all zones build and test correctly."
echo ""
