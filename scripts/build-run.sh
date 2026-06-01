#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
MODE="${1:-all}"

if [ "$#" -gt 0 ]; then
    shift
fi

case "$MODE" in
    compiler)
        sh "$ROOT_DIR/scripts/compiler/build.sh"
        ;;
    ide)
        sh "$ROOT_DIR/scripts/ide/build.sh"
        ;;
    all)
        sh "$ROOT_DIR/scripts/compiler/build.sh"
        sh "$ROOT_DIR/scripts/ide/build.sh"
        ;;
    *)
        echo "Usage: sh scripts/build-run.sh [compiler | ide | all]" >&2
        exit 2
        ;;
esac
