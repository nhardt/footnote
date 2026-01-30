#!/bin/bash

set -e

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <input-icon-1024.png>"
    echo "Example: $0 footnote-icon.png"
    exit 1
fi

INPUT="$1"

if [ ! -f "$INPUT" ]; then
    echo "Error: Input file '$INPUT' not found"
    exit 1
fi

if ! command -v convert &> /dev/null; then
    echo "Error: ImageMagick is required. Install with:"
    echo "  macOS: brew install imagemagick"
    echo "  Ubuntu/Debian: sudo apt install imagemagick"
    echo "  Fedora/RHEL: sudo dnf install ImageMagick"
    exit 1
fi

echo "Generating icons from $INPUT..."

OUTPUT_DIR="generated-icons"
mkdir -p "$OUTPUT_DIR"/{linux,windows,macos,ios,android}

echo "Generating Linux icons..."
for size in 16 32 48 64 128 256 512; do
    convert "$INPUT" -resize ${size}x${size} "$OUTPUT_DIR/linux/icon-${size}.png"
done

echo "Generating Windows .ico..."
convert "$INPUT" -define icon:auto-resize=256,128,96,64,48,32,16 "$OUTPUT_DIR/windows/icon.ico"

echo "Generating macOS icons..."
ICONSET_DIR="$OUTPUT_DIR/macos/icon.iconset"
mkdir -p "$ICONSET_DIR"

convert "$INPUT" -resize 16x16     "$ICONSET_DIR/icon_16x16.png"
convert "$INPUT" -resize 32x32     "$ICONSET_DIR/icon_16x16@2x.png"
convert "$INPUT" -resize 32x32     "$ICONSET_DIR/icon_32x32.png"
convert "$INPUT" -resize 64x64     "$ICONSET_DIR/icon_32x32@2x.png"
convert "$INPUT" -resize 128x128   "$ICONSET_DIR/icon_128x128.png"
convert "$INPUT" -resize 256x256   "$ICONSET_DIR/icon_128x128@2x.png"
convert "$INPUT" -resize 256x256   "$ICONSET_DIR/icon_256x256.png"
convert "$INPUT" -resize 512x512   "$ICONSET_DIR/icon_256x256@2x.png"
convert "$INPUT" -resize 512x512   "$ICONSET_DIR/icon_512x512.png"
convert "$INPUT" -resize 1024x1024 "$ICONSET_DIR/icon_512x512@2x.png"

if command -v iconutil &> /dev/null; then
    echo "Creating .icns with iconutil (macOS only)..."
    iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_DIR/macos/icon.icns"
    rm -rf "$ICONSET_DIR"
elif command -v png2icns &> /dev/null; then
    echo "Creating .icns with png2icns..."
    png2icns "$OUTPUT_DIR/macos/icon.icns" "$ICONSET_DIR"/*.png
    rm -rf "$ICONSET_DIR"
else
    echo "Warning: Neither 'iconutil' nor 'png2icns' found. Keeping .iconset directory."
    echo "  On macOS, run: iconutil -c icns $ICONSET_DIR -o $OUTPUT_DIR/macos/icon.icns"
    echo "  Or install png2icns: brew install png2icns"
fi

echo "Generating iOS icons..."
IOS_SIZES=(20 29 40 58 60 76 80 87 120 152 167 180 1024)
for size in "${IOS_SIZES[@]}"; do
    convert "$INPUT" -resize ${size}x${size} "$OUTPUT_DIR/ios/icon-${size}.png"
done

echo "Generating Android icons..."
mkdir -p "$OUTPUT_DIR/android"/{mipmap-mdpi,mipmap-hdpi,mipmap-xhdpi,mipmap-xxhdpi,mipmap-xxxhdpi}

convert "$INPUT" -resize 48x48   "$OUTPUT_DIR/android/mipmap-mdpi/ic_launcher.png"
convert "$INPUT" -resize 72x72   "$OUTPUT_DIR/android/mipmap-hdpi/ic_launcher.png"
convert "$INPUT" -resize 96x96   "$OUTPUT_DIR/android/mipmap-xhdpi/ic_launcher.png"
convert "$INPUT" -resize 144x144 "$OUTPUT_DIR/android/mipmap-xxhdpi/ic_launcher.png"
convert "$INPUT" -resize 192x192 "$OUTPUT_DIR/android/mipmap-xxxhdpi/ic_launcher.png"

convert "$INPUT" -resize 108x108 "$OUTPUT_DIR/android/ic_launcher_adaptive.png"

echo ""
echo "✓ Icon generation complete!"
echo ""
echo "Generated files:"
echo "  Linux:   $OUTPUT_DIR/linux/"
echo "  Windows: $OUTPUT_DIR/windows/icon.ico"
echo "  macOS:   $OUTPUT_DIR/macos/"
echo "  iOS:     $OUTPUT_DIR/ios/"
echo "  Android: $OUTPUT_DIR/android/"
echo ""
echo "Next steps:"
echo "  - Copy files to your project's icon directories"
echo "  - For Android adaptive icons, you may need foreground/background layers"
echo "  - For iOS, import icons into Assets.xcassets"
echo "  - Update Tauri/Dioxus config files with new icon paths"
