#!/bin/bash
# Trinity Genesis - Source Dump Generator
# Generates a comprehensive source dump and compares it to the previous version

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DUMP_FILE="$PROJECT_DIR/TRINITY_SOURCE_DUMP.rs"
BACKUP_FILE="$PROJECT_DIR/TRINITY_SOURCE_DUMP.rs.bak"
DIFF_FILE="$PROJECT_DIR/TRINITY_SOURCE_DUMP.diff"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         TRINITY GENESIS - SOURCE DUMP GENERATOR              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Backup existing dump if it exists
if [ -f "$DUMP_FILE" ]; then
    echo "📦 Backing up previous dump..."
    cp "$DUMP_FILE" "$BACKUP_FILE"
    PREV_LINES=$(wc -l < "$BACKUP_FILE")
    echo "   Previous dump: $PREV_LINES lines"
fi

# Generate header with system context
echo "🔧 Generating new source dump..."
cat > "$DUMP_FILE" << 'HEADER'
// ============================================================================
// TRINITY GENESIS SOURCE DUMP
// Generated: TIMESTAMP_PLACEHOLDER
// ============================================================================
// 
// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║  SYSTEM CONTEXT (READ BEFORE MODIFYING):                                  ║
// ╠═══════════════════════════════════════════════════════════════════════════╣
// ║  Hardware: AMD Strix Halo (Ryzen AI Max+ 395)                             ║
// ║  RAM: 128GB DDR5 Unified Memory                                           ║
// ║  GPU VRAM: Up to 124GB (via GTT kernel override)                          ║
// ║  HSA Version: 11.5.1 (gfx1151)                                            ║
// ╠═══════════════════════════════════════════════════════════════════════════╣
// ║  VERIFIED MODELS:                                                          ║
// ║  - GLM-4.6V-265B (116GB) ✅ WORKS                                          ║
// ║  - Qwen3-235B (105GB) ✅ WORKS                                              ║
// ║  - Llama-4-Scout-17B (64GB) ✅ WORKS                                        ║
// ║  - Qwen3-Next-80B (46GB) ✅ WORKS                                           ║
// ╠═══════════════════════════════════════════════════════════════════════════╣
// ║  ⚠️  DO NOT assume memory limits from `free -h` output!                    ║
// ║  📖  See: docs/HARDWARE_CONTEXT.md                                         ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
//
// This file is auto-generated. Run: ./scripts/generate_source_dump.sh
// ============================================================================

HEADER

# Replace timestamp
sed -i "s/TIMESTAMP_PLACEHOLDER/$(date -Iseconds)/" "$DUMP_FILE"

# Collect all Rust source files from crates
echo "📂 Scanning crates..."
for crate_dir in "$PROJECT_DIR/crates"/*; do
    if [ -d "$crate_dir/src" ]; then
        crate_name=$(basename "$crate_dir")
        echo "   Processing: $crate_name"
        
        find "$crate_dir/src" -name "*.rs" -type f | sort | while read -r src_file; do
            rel_path="${src_file#$PROJECT_DIR/}"
            echo "" >> "$DUMP_FILE"
            echo "// ============ $rel_path ============" >> "$DUMP_FILE"
            cat "$src_file" >> "$DUMP_FILE"
        done
    fi
done

NEW_LINES=$(wc -l < "$DUMP_FILE")
echo ""
echo "✅ New dump generated: $NEW_LINES lines"

# Generate diff if backup exists
if [ -f "$BACKUP_FILE" ]; then
    echo ""
    echo "📊 Comparing with previous dump..."
    
    if diff -u "$BACKUP_FILE" "$DUMP_FILE" > "$DIFF_FILE" 2>/dev/null; then
        echo "   No changes detected."
        rm -f "$DIFF_FILE"
    else
        DIFF_LINES=$(wc -l < "$DIFF_FILE")
        ADDED=$(grep -c "^+" "$DIFF_FILE" 2>/dev/null || echo 0)
        REMOVED=$(grep -c "^-" "$DIFF_FILE" 2>/dev/null || echo 0)
        
        echo "   Changes detected!"
        echo "   - Diff file: $DIFF_FILE ($DIFF_LINES lines)"
        echo "   - Lines added: ~$ADDED"
        echo "   - Lines removed: ~$REMOVED"
        echo ""
        echo "📝 Key changes (first 20 lines of diff):"
        echo "────────────────────────────────────────"
        head -20 "$DIFF_FILE" | grep -E "^[+-].*\.(rs|md)" | head -10 || echo "   (file-level diff only)"
    fi
fi

echo ""
echo "Done! Files:"
echo "  📄 $DUMP_FILE"
[ -f "$BACKUP_FILE" ] && echo "  📦 $BACKUP_FILE (previous)"
[ -f "$DIFF_FILE" ] && echo "  📊 $DIFF_FILE (changes)"
