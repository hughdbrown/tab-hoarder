#!/bin/bash
set -e

echo "🔨 Building Tab Hoarder Chrome Extension..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ Error: wasm-pack is not installed."
    echo "📦 Install with: cargo install wasm-pack"
    exit 1
fi

# Build the Rust code to WASM
echo "📦 Compiling Rust to WASM..."
wasm-pack build --target web --out-dir pkg --release

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo "✅ Build complete!"
    echo ""
    echo "📋 Next steps:"
    echo "  1. Open Chrome and go to chrome://extensions/"
    echo "  2. Enable 'Developer mode' (top right)"
    echo "  3. Click 'Load unpacked'"
    echo "  4. Select this directory: $(pwd)"
    echo ""
    echo "🔄 After making changes:"
    echo "  - Run: ./build.sh"
    echo "  - Click reload icon in Chrome extensions page"
else
    echo "❌ Build failed!"
    exit 1
fi
