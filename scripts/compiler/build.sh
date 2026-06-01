#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/dist/compiler"

cd "$ROOT_DIR"

cargo build --release --bin peps
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cp target/release/peps "$OUT_DIR/peps!"

cat > "$OUT_DIR/linux.sh" <<'LAUNCHER'
#!/usr/bin/env sh
set -eu

DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$DIR/peps!" "$@"
LAUNCHER

cp "$OUT_DIR/linux.sh" "$OUT_DIR/peps.sh"

chmod +x "$OUT_DIR/peps!" "$OUT_DIR/linux.sh" "$OUT_DIR/peps.sh"

echo "Built Peps compiler runner: dist/compiler"
echo "Run it with: './dist/compiler/linux.sh' path/to/file.peps"
