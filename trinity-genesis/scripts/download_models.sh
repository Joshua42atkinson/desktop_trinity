#!/bin/bash
# Trinity Model Download Script
# Downloads all required models for full Trinity functionality
set -e

TRINITY_MODELS="${HOME}/.local/share/trinity/models"
echo "📦 Trinity Model Downloads"
echo "=========================="
echo "Target directory: $TRINITY_MODELS"
echo ""

# Create base directory
mkdir -p "$TRINITY_MODELS"

# ============================================================================
# Phase 1: Piper TTS Voices (~500MB)
# ============================================================================
echo "🔊 Phase 1: Downloading Piper TTS voices..."
PIPER_DIR="$TRINITY_MODELS/piper"
mkdir -p "$PIPER_DIR"

# Amy - Warm, professional US English
if [ ! -f "$PIPER_DIR/en_US-amy-medium.onnx" ]; then
    echo "   Downloading en_US-amy-medium..."
    wget -q --show-progress -O "$PIPER_DIR/en_US-amy-medium.onnx" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/medium/en_US-amy-medium.onnx"
    wget -q -O "$PIPER_DIR/en_US-amy-medium.onnx.json" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/medium/en_US-amy-medium.onnx.json"
else
    echo "   ✅ en_US-amy-medium already downloaded"
fi

# Lessac - High quality US English
if [ ! -f "$PIPER_DIR/en_US-lessac-high.onnx" ]; then
    echo "   Downloading en_US-lessac-high..."
    wget -q --show-progress -O "$PIPER_DIR/en_US-lessac-high.onnx" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/high/en_US-lessac-high.onnx"
    wget -q -O "$PIPER_DIR/en_US-lessac-high.onnx.json" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/high/en_US-lessac-high.onnx.json"
else
    echo "   ✅ en_US-lessac-high already downloaded"
fi

echo "   ✅ Piper voices complete"
echo ""

# ============================================================================
# Phase 2: SDXL Turbo (~7GB)
# ============================================================================
echo "🎨 Phase 2: Downloading SDXL Turbo..."
SDXL_DIR="$TRINITY_MODELS/sdxl"
mkdir -p "$SDXL_DIR"

# SDXL Turbo main model (FP16) - use wget directly
if [ ! -f "$SDXL_DIR/sd_xl_turbo_1.0_fp16.safetensors" ]; then
    echo "   Downloading SDXL Turbo FP16 (6.94GB)..."
    wget -q --show-progress -O "$SDXL_DIR/sd_xl_turbo_1.0_fp16.safetensors" \
        "https://huggingface.co/stabilityai/sdxl-turbo/resolve/main/sd_xl_turbo_1.0_fp16.safetensors"
else
    echo "   ✅ SDXL Turbo already downloaded"
fi

echo "   ✅ SDXL Turbo complete"
echo ""

# ============================================================================
# Phase 3: Cosmos 7B (~30GB) - Optional, comment out if not needed
# ============================================================================
echo "🌍 Phase 3: Downloading Cosmos 7B Diffusion..."
COSMOS_DIR="$TRINITY_MODELS/cosmos"
mkdir -p "$COSMOS_DIR"

if [ ! -d "$COSMOS_DIR/diffusion-7b" ] || [ -z "$(ls -A $COSMOS_DIR/diffusion-7b 2>/dev/null)" ]; then
    echo "   Downloading Cosmos 7B Text2World (~30GB, this will take a while)..."
    huggingface-cli download nvidia/Cosmos-1.0-Diffusion-7B-Text2World \
        --local-dir "$COSMOS_DIR/diffusion-7b" \
        --local-dir-use-symlinks False || {
        echo "   ⚠️ Cosmos download failed (may require NGC account or login)"
        echo "   You can try: huggingface-cli login"
    }
else
    echo "   ✅ Cosmos 7B already downloaded"
fi

echo ""

# ============================================================================
# Phase 4: TRELLIS 3D (~8GB)
# ============================================================================
echo "🎮 Phase 4: Downloading TRELLIS 2-4B..."
TRELLIS_DIR="$TRINITY_MODELS/trellis"
mkdir -p "$TRELLIS_DIR"

if [ ! -d "$TRELLIS_DIR/trellis-2-4b" ] || [ -z "$(ls -A $TRELLIS_DIR/trellis-2-4b 2>/dev/null)" ]; then
    echo "   Downloading TRELLIS 2-4B (~8GB)..."
    huggingface-cli download microsoft/TRELLIS.2-4B \
        --local-dir "$TRELLIS_DIR/trellis-2-4b" \
        --local-dir-use-symlinks False || {
        echo "   ⚠️ TRELLIS download failed"
    }
else
    echo "   ✅ TRELLIS already downloaded"
fi

echo ""

# ============================================================================
# Summary
# ============================================================================
echo "=========================="
echo "📊 Download Summary"
echo "=========================="
echo ""
echo "Model locations:"
echo "  Piper TTS:  $PIPER_DIR"
echo "  SDXL Turbo: $SDXL_DIR"
echo "  Cosmos:     $COSMOS_DIR"
echo "  TRELLIS:    $TRELLIS_DIR"
echo ""
echo "Total size:"
du -sh "$TRINITY_MODELS" 2>/dev/null || echo "  (calculating...)"
echo ""
echo "✅ Model downloads complete!"
