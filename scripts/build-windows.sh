#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TARGET="${PEPS_WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
COMPILER_OUT="$ROOT_DIR/dist/compiler/windows"
IDE_OUT="$ROOT_DIR/dist/ide/windows"
WINDOWS_TARGET_ROOT="${PEPS_WINDOWS_TARGET_DIR:-${TMPDIR:-/tmp}/peps-windows-target}"
TARGET_RELEASE_DIR="$WINDOWS_TARGET_ROOT/$TARGET/release"

cd "$ROOT_DIR"

VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
if [ -z "$VERSION" ]; then
    echo "error: could not read package version from Cargo.toml" >&2
    exit 1
fi
COMPILER_NAME="peps-$VERSION.exe"
COMPILER_LAUNCHER="peps-$VERSION.cmd"
IDE_NAME="peps-ide-$VERSION.exe"
IDE_LAUNCHER="peps-ide-$VERSION.cmd"

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

TARGET_LIBDIR=$(rustc --print target-libdir --target "$TARGET" 2>/dev/null || true)
if [ -z "$TARGET_LIBDIR" ] || [ ! -d "$TARGET_LIBDIR" ] || ! find "$TARGET_LIBDIR" -maxdepth 1 -name 'libcore-*.rlib' | grep -q .; then
    echo "error: Rust target '$TARGET' is not installed." >&2
    echo "Install it, then run this script again." >&2
    if command -v rustup >/dev/null 2>&1; then
        echo "rustup: rustup target add $TARGET" >&2
    else
        echo "rustup is not installed, so your Rust likely came from your distro packages." >&2
        echo "Fedora: sudo dnf install rust-std-static-x86_64-pc-windows-gnu" >&2
        echo "Alternative: install rustup from https://rustup.rs/ and then run:" >&2
        echo "  rustup target add $TARGET" >&2
    fi
    exit 1
fi

export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
export CARGO_TARGET_DIR="$WINDOWS_TARGET_ROOT"

if [ ! -f Cargo.toml ]; then
    echo "error: Cargo.toml not found at project root: $ROOT_DIR" >&2
    exit 1
fi

if [ ! -d ide ]; then
    echo "error: ide/ directory not found at project root: $ROOT_DIR" >&2
    exit 1
fi

cargo build --release --bin peps --target "$TARGET"

(cd ide && pnpm install --frozen-lockfile && pnpm run build)

if [ ! -f ide/dist/index.html ]; then
    echo "error: frontend build did not produce ide/dist/index.html" >&2
    exit 1
fi

cargo build --release --bin peps-ide --target "$TARGET"

rm -rf "$COMPILER_OUT" "$IDE_OUT"
mkdir -p "$COMPILER_OUT" "$IDE_OUT/frontend"

cp "$TARGET_RELEASE_DIR/peps.exe" "$COMPILER_OUT/$COMPILER_NAME"
cp "$TARGET_RELEASE_DIR/peps-ide.exe" "$IDE_OUT/$IDE_NAME"
cp -R ide/dist "$IDE_OUT/frontend/dist"

cat > "$COMPILER_OUT/$COMPILER_LAUNCHER" <<CMD
@echo off
set DIR=%~dp0
"%DIR%$COMPILER_NAME" %*
CMD

cat > "$IDE_OUT/$IDE_LAUNCHER" <<CMD
@echo off
set DIR=%~dp0
cd /d "%DIR%"
"%DIR%$IDE_NAME" %*
CMD

echo "Built Peps Windows dists from Linux:"
echo "  dist/compiler/windows/$COMPILER_NAME"
echo "  dist/ide/windows/$IDE_NAME"
echo "Version: $VERSION"
echo "Windows Cargo target cache: $WINDOWS_TARGET_ROOT"
