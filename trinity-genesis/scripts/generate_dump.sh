#!/bin/bash
# Generate a complete source dump of the Trinity Genesis codebase
# Usage: ./scripts/generate_dump.sh

OUTPUT="TRINITY_GENESIS_SOURCE_DUMP_V2.md"
echo "# Trinity Genesis Source Dump" > $OUTPUT
echo "Generated: $(date)" >> $OUTPUT
echo "" >> $OUTPUT

echo "Scanning files..."

find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.sh" \) \
    -not -path "*/target/*" \
    -not -path "*/.git/*" \
    -not -path "*/assets/*" \
    -not -path "*/node_modules/*" \
    -not -path "*/$OUTPUT" \
    -not -name "TRINITY_SOURCE_DUMP.rs" \
    -not -name "trinity_genesis_source_dump.md" \
    | sort | while read -r file; do
    
    echo "Processing $file..."
    echo "## File: $file" >> $OUTPUT
    
    # Determine extension for markdown fence
    EXT="${file##*.}"
    if [ "$EXT" == "rs" ]; then LANG="rust"; 
    elif [ "$EXT" == "toml" ]; then LANG="toml";
    elif [ "$EXT" == "sh" ]; then LANG="bash";
    elif [ "$EXT" == "md" ]; then LANG="markdown";
    else LANG=""; fi
    
    echo "\`\`\`$LANG" >> $OUTPUT
    cat "$file" >> $OUTPUT
    echo "" >> $OUTPUT
    echo "\`\`\`" >> $OUTPUT
    echo "" >> $OUTPUT
    echo "---" >> $OUTPUT
    echo "" >> $OUTPUT
done

echo "✅ Source dump generated at: $(pwd)/$OUTPUT"
du -h $OUTPUT
