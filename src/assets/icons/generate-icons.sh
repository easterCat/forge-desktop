#!/bin/bash
# Icon Generation Script for Forge
# This script generates all required icon formats from SVG sources
#
# Prerequisites:
#   macOS: brew install imagemagick librsvg
#   Linux: apt-get install imagemagick librsvg
#   Or use Node.js: npm install -g @resvg/cli
#
# Usage: ./generate-icons.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ICONS_DIR="$SCRIPT_DIR/../src-tauri/icons"
DESIGN_ICONS_DIR="$SCRIPT_DIR"

echo "🎨 Forge Icon Generator"
echo "========================"

# Check for tools
if command -v rsvg-convert &> /dev/null; then
    RSVG_CMD="rsvg-convert"
    echo "✓ Using rsvg-convert"
elif command -v convert &> /dev/null; then
    RSVG_CMD="convert"
    echo "✓ Using ImageMagick"
elif command -v resvg &> /dev/null; then
    RSVG_CMD="resvg"
    echo "✓ Using resvg"
else
    echo "❌ No SVG converter found."
    echo "   Please install one of:"
    echo "   - ImageMagick: brew install imagemagick librsvg"
    echo "   - resvg: npm install -g @resvg/cli"
    exit 1
fi

# Function to convert SVG to PNG
convert_icon() {
    local src="$1"
    local dest="$2"
    local size="$3"

    if [ -f "$src" ]; then
        echo "  Converting $src -> $dest ($size)"
        case "$RSVG_CMD" in
            rsvg-convert)
                rsvg-convert -w "$size" -h "$size" "$src" -o "$dest"
                ;;
            convert)
                convert -background none -resize "${size}x${size}" "$src" "$dest"
                ;;
            resvg)
                resvg "$src" "$dest" --width "$size" --height "$size"
                ;;
        esac
    else
        echo "  ⚠ Source not found: $src"
    fi
}

# Generate App Icons for Tauri bundle
echo ""
echo "📱 Generating App Icons..."

# 32x32
convert_icon "$DESIGN_ICONS_DIR/app/icon-app-base.svg" "$ICONS_DIR/32x32.png" 32

# 128x128
convert_icon "$DESIGN_ICONS_DIR/app/icon-app-base.svg" "$ICONS_DIR/128x128.png" 128

# 256x256 (for 128x128@2x)
convert_icon "$DESIGN_ICONS_DIR/app/icon-app-base.svg" "$ICONS_DIR/128x128@2x.png" 256

echo ""
echo "✅ Icon generation complete!"
echo ""
echo "📁 Generated files:"
ls -la "$ICONS_DIR/" 2>/dev/null || echo "Icons directory not found"
