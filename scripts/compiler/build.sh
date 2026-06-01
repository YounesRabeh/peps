#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/dist/compiler/linux"
APPDIR="$OUT_DIR/PepsCompiler.AppDir"
APPIMAGE="$OUT_DIR/peps-compiler-x86_64.AppImage"
TMP_ROOT="${TMPDIR:-/tmp}/peps-compiler-appimage-$$"
TMP_APPDIR="$TMP_ROOT/PepsCompiler.AppDir"
TMP_APPIMAGE="$TMP_ROOT/peps-compiler-x86_64.AppImage"
APPIMAGETOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
trap 'rm -rf "$TMP_ROOT"' EXIT

cd "$ROOT_DIR"

cargo build --release --bin peps
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR" "$TMP_APPDIR/usr/bin"

cp target/release/peps "$OUT_DIR/peps!"
cp target/release/peps "$OUT_DIR/peps!-bytecode"
cp target/release/peps "$TMP_APPDIR/usr/bin/peps!"

cat > "$OUT_DIR/linux.sh" <<'LAUNCHER'
#!/usr/bin/env sh
set -eu

DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$DIR/peps!" "$@"
LAUNCHER

cat > "$TMP_APPDIR/AppRun" <<'APPRUN'
#!/usr/bin/env sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$HERE/usr/bin/peps!" "$@"
APPRUN

cat > "$TMP_APPDIR/peps-compiler.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Peps Compiler
Exec=peps!
Icon=peps
Categories=Development;
Terminal=true
DESKTOP

cp ide/public/favicon.svg "$TMP_APPDIR/peps.svg"
cp -R "$TMP_APPDIR" "$APPDIR"

chmod +x "$OUT_DIR/peps!" "$OUT_DIR/peps!-bytecode" "$OUT_DIR/linux.sh" "$TMP_APPDIR/AppRun" "$TMP_APPDIR/usr/bin/peps!"
chmod +x "$APPDIR/AppRun" "$APPDIR/usr/bin/peps!"

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
echo "Manual run: './dist/compiler/linux/linux.sh' path/to/file.peps"
