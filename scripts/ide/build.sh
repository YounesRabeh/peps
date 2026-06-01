#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/dist/ide/linux"
APPDIR="$OUT_DIR/PepsIDE.AppDir"
APPIMAGE="$OUT_DIR/peps-ide-x86_64.AppImage"
TMP_ROOT="${TMPDIR:-/tmp}/peps-ide-appimage-$$"
TMP_APPDIR="$TMP_ROOT/PepsIDE.AppDir"
TMP_APPIMAGE="$TMP_ROOT/peps-ide-x86_64.AppImage"
APPIMAGETOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
trap 'rm -rf "$TMP_ROOT"' EXIT

cd "$ROOT_DIR"

if [ ! -d ide ]; then
    echo "error: ide/ directory not found" >&2
    exit 1
fi

if [ ! -f Cargo.toml ]; then
    echo "error: Cargo.toml not found at project root" >&2
    exit 1
fi

if [ -f ide/package-lock.json ]; then
    (cd ide && npm ci && npm run build)
else
    (cd ide && npm install && npm run build)
fi

cargo build --release --bin peps-ide

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR" "$TMP_APPDIR/usr/bin" "$TMP_APPDIR/usr/bin/frontend"

cp target/release/peps-ide "$TMP_APPDIR/usr/bin/peps-ide"
cp -R ide/dist "$TMP_APPDIR/usr/bin/frontend/dist"

cat > "$TMP_APPDIR/AppRun" <<'APPRUN'
#!/usr/bin/env sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$HERE/usr/bin"
exec "$HERE/usr/bin/peps-ide" "$@"
APPRUN

cat > "$TMP_APPDIR/peps-ide.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Peps IDE
Exec=peps-ide
Icon=peps
Categories=Development;
Terminal=false
DESKTOP

cp ide/public/favicon.svg "$TMP_APPDIR/peps.svg"
cp -R "$TMP_APPDIR" "$APPDIR"

chmod +x "$TMP_APPDIR/AppRun" "$TMP_APPDIR/usr/bin/peps-ide"
chmod +x "$APPDIR/AppRun" "$APPDIR/usr/bin/peps-ide"

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

echo "Built Peps IDE Linux dist: dist/ide/linux"
echo "Manual start: ./dist/ide/linux/peps-ide-x86_64.AppImage"
