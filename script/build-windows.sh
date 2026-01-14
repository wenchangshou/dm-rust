#!/bin/bash
# Build Windows executable from Linux
# This script cross-compiles the Rust project for Windows

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET="x86_64-pc-windows-gnu"
OUTPUT_DIR="$PROJECT_DIR/target/$TARGET/release"

echo "=== Building dm-rust for Windows ==="
echo "Project directory: $PROJECT_DIR"
echo "Target: $TARGET"

# Check if cross-compilation toolchain is installed
check_toolchain() {
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "Installing Windows target..."
        rustup target add "$TARGET"
    fi

    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo "Error: mingw-w64 is not installed."
        echo "Please install it with:"
        echo "  Ubuntu/Debian: sudo apt install mingw-w64"
        echo "  Fedora: sudo dnf install mingw64-gcc"
        echo "  Arch: sudo pacman -S mingw-w64-gcc"
        exit 1
    fi
}

# Build the project
build() {
    cd "$PROJECT_DIR"

    echo ""
    echo "Building release..."
    cargo build --release  --features swagger  --target "$TARGET"

    if [ $? -eq 0 ]; then
        echo ""
        echo "=== Build successful ==="
        echo "Executable: $OUTPUT_DIR/dm-rust.exe"

        # Show file size
        if [ -f "$OUTPUT_DIR/dm-rust.exe" ]; then
            SIZE=$(du -h "$OUTPUT_DIR/dm-rust.exe" | cut -f1)
            echo "Size: $SIZE"
        fi
    else
        echo "Build failed!"
        exit 1
    fi
}

# Optional: Create a distribution package
package() {
    DIST_DIR="$PROJECT_DIR/dist/windows"
    mkdir -p "$DIST_DIR"

    cp "$OUTPUT_DIR/dm-rust.exe" "$DIST_DIR/"
    cp "$PROJECT_DIR/config.example.json" "$DIST_DIR/config.json"

    echo ""
    echo "Distribution package created: $DIST_DIR"
    ls -la "$DIST_DIR"
}

# Main
check_toolchain
build

if [ "$1" == "--package" ]; then
    package
fi

echo ""
echo "Done!"
