#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
OUT_DIR="$ROOT_DIR/dist/compiler"

cd "$ROOT_DIR"

cargo build --release --bin peps
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cp target/release/peps "$OUT_DIR/peps!"

cat > "$OUT_DIR/peps.sh" <<'LAUNCHER'
#!/usr/bin/env sh
set -eu

DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$DIR/peps!" "$@"
LAUNCHER

chmod +x "$OUT_DIR/peps!" "$OUT_DIR/peps.sh"

echo "Built Peps compiler runner: dist/compiler"
echo "Run it with: './dist/compiler/peps!' path/to/file.peps"
