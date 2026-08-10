#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

cd "$ROOT_DIR"

cargo fmt --all --check
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings

(cd ide && pnpm install --frozen-lockfile && pnpm test -- --run)

sh "$ROOT_DIR/scripts/devcontainer/build.sh"
sh "$ROOT_DIR/scripts/build-run.sh" all
sh "$ROOT_DIR/scripts/build-windows.sh"

echo "Passed checks and built the local development container and Linux and Windows compiler and IDE artifacts."
