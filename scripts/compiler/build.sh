#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/dist/compiler/linux"
APPDIR="$OUT_DIR/PepsCompiler.AppDir"
TMP_ROOT="${TMPDIR:-/tmp}/peps-compiler-appimage-$$"
TMP_APPDIR="$TMP_ROOT/PepsCompiler.AppDir"
APPIMAGETOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
trap 'rm -rf "$TMP_ROOT"' EXIT

cd "$ROOT_DIR"

VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
if [ -z "$VERSION" ]; then
    echo "error: could not read package version from Cargo.toml" >&2
    exit 1
fi

CLI_BINARY="peps-$VERSION"
BYTECODE_BINARY="peps-bytecode-$VERSION"
LAUNCHER="linux-$VERSION.sh"
APPIMAGE="$OUT_DIR/peps-compiler-$VERSION-x86_64.AppImage"
TMP_APPIMAGE="$TMP_ROOT/peps-compiler-$VERSION-x86_64.AppImage"

cargo build --release --bin peps
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR" "$TMP_APPDIR/usr/bin"

cp target/release/peps "$OUT_DIR/$CLI_BINARY"
cp target/release/peps "$OUT_DIR/$BYTECODE_BINARY"
cp target/release/peps "$TMP_APPDIR/usr/bin/peps"

cat > "$OUT_DIR/$LAUNCHER" <<LAUNCHER
#!/usr/bin/env sh
set -eu

DIR=\$(CDPATH= cd -- "\$(dirname -- "\$0")" && pwd)
exec "\$DIR/$CLI_BINARY" "\$@"
LAUNCHER

cat > "$TMP_APPDIR/AppRun" <<'APPRUN'
#!/usr/bin/env sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$HERE/usr/bin/peps" "$@"
APPRUN

cat > "$TMP_APPDIR/peps-compiler.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Peps Compiler
Exec=peps
Icon=peps
Categories=Development;
Terminal=true
DESKTOP

cp ide/public/favicon.svg "$TMP_APPDIR/peps.svg"
cp -R "$TMP_APPDIR" "$APPDIR"

chmod +x "$OUT_DIR/$CLI_BINARY" "$OUT_DIR/$BYTECODE_BINARY" "$OUT_DIR/$LAUNCHER" "$TMP_APPDIR/AppRun" "$TMP_APPDIR/usr/bin/peps"
chmod +x "$APPDIR/AppRun" "$APPDIR/usr/bin/peps"

APPIMAGETOOL="${APPIMAGETOOL:-}"
if [ -z "$APPIMAGETOOL" ]; then
    if command -v appimagetool >/dev/null 2>&1; then
        APPIMAGETOOL=$(command -v appimagetool)
    else
        APPIMAGETOOL="$ROOT_DIR/.tools/appimagetool-x86_64.AppImage"
        if [ ! -x "$APPIMAGETOOL" ]; then
            mkdir -p "$ROOT_DIR/.tools"
            curl -L "$APPIMAGETOOL_URL" -o "$APPIMAGETOOL"
            chmod +x "$APPIMAGETOOL"
        fi
    fi
fi

APPIMAGE_LOG="$TMP_ROOT/appimagetool.log"
if ! ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGETOOL" --no-appstream "$TMP_APPDIR" "$TMP_APPIMAGE" >"$APPIMAGE_LOG" 2>&1; then
    cat "$APPIMAGE_LOG" >&2
    exit 1
fi
mv "$TMP_APPIMAGE" "$APPIMAGE"
chmod +x "$APPIMAGE"

echo "Built Peps compiler Linux dist: dist/compiler/linux"
echo "Version: $VERSION"
echo "Manual run: './dist/compiler/linux/$LAUNCHER' path/to/file.peps"
