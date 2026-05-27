#!/usr/bin/env sh
set -eu

cargo build --release
mkdir -p dist
rm -f dist/peps
cp target/release/peps 'dist/peps!'

echo "Built standalone runner: dist/peps!"
echo "Run it with: './dist/peps!' path/to/file.peps"
