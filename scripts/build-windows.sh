#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TARGET="${PEPS_WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
COMPILER_OUT="$ROOT_DIR/dist/compiler/windows"
IDE_OUT="$ROOT_DIR/dist/ide/windows"
TARGET_RELEASE_DIR="$ROOT_DIR/target/$TARGET/release"

cd "$ROOT_DIR"

if [ "$TARGET" != "x86_64-pc-windows-gnu" ]; then
    echo "error: Linux cross-builds should use PEPS_WINDOWS_TARGET=x86_64-pc-windows-gnu" >&2
    echo "Current target: $TARGET" >&2
    exit 1
fi

if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "error: x86_64-w64-mingw32-gcc was not found." >&2
    echo "Install the MinGW Windows compiler, then run this script again." >&2
    echo "Fedora: sudo dnf install mingw64-gcc" >&2
    echo "Ubuntu/Debian: sudo apt install gcc-mingw-w64-x86-64" >&2
    exit 1
fi

if ! rustc --print target-libdir --target "$TARGET" >/dev/null 2>&1; then
    echo "error: Rust target '$TARGET' is not installed." >&2
    echo "Install it, then run this script again:" >&2
    echo "  rustup target add $TARGET" >&2
    exit 1
fi

export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

if [ ! -f Cargo.toml ]; then
    echo "error: Cargo.toml not found at project root: $ROOT_DIR" >&2
    exit 1
fi

if [ ! -d ide ]; then
    echo "error: ide/ directory not found at project root: $ROOT_DIR" >&2
    exit 1
fi

cargo build --release --bin peps --target "$TARGET"

if [ -f ide/package-lock.json ]; then
    (cd ide && npm ci && npm run build)
else
    (cd ide && npm install && npm run build)
fi

if [ ! -f ide/dist/index.html ]; then
    echo "error: frontend build did not produce ide/dist/index.html" >&2
    exit 1
fi

cargo build --release --bin peps-ide --target "$TARGET"

rm -rf "$COMPILER_OUT" "$IDE_OUT"
mkdir -p "$COMPILER_OUT" "$IDE_OUT/frontend"

cp "$TARGET_RELEASE_DIR/peps.exe" "$COMPILER_OUT/peps!.exe"
cp "$TARGET_RELEASE_DIR/peps-ide.exe" "$IDE_OUT/peps-ide.exe"
cp -R ide/dist "$IDE_OUT/frontend/dist"

cat > "$COMPILER_OUT/peps.cmd" <<'CMD'
@echo off
set DIR=%~dp0
"%DIR%peps!.exe" %*
CMD

cat > "$IDE_OUT/peps-ide.cmd" <<'CMD'
@echo off
set DIR=%~dp0
cd /d "%DIR%"
"%DIR%peps-ide.exe" %*
CMD

echo "Built Peps Windows dists from Linux:"
echo "  dist/compiler/windows/peps!.exe"
echo "  dist/ide/windows/peps-ide.exe"
