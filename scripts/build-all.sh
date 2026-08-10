#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

sh "$ROOT_DIR/scripts/build-run.sh" all
sh "$ROOT_DIR/scripts/build-windows.sh"

echo "Built Linux and Windows compiler and IDE artifacts under dist/."
