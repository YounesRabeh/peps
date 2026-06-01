#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/dist/ide"

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
mkdir -p "$OUT_DIR/frontend"

cp target/release/peps-ide "$OUT_DIR/peps-ide"
cp -R ide/dist "$OUT_DIR/frontend/dist"

cat > "$OUT_DIR/linux.sh" <<'LAUNCHER'
#!/usr/bin/env sh
set -eu

DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$DIR"
exec "$DIR/peps-ide" "$@"
LAUNCHER

cp "$OUT_DIR/linux.sh" "$OUT_DIR/peps-ide.sh"

chmod +x "$OUT_DIR/peps-ide" "$OUT_DIR/linux.sh" "$OUT_DIR/peps-ide.sh"

echo "Built Peps IDE server and frontend: dist/ide"
echo "Start the IDE with: ./dist/ide/linux.sh"
