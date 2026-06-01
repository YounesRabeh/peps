#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
MODE="${1:-all}"

if [ "$#" -gt 0 ]; then
    shift
fi

case "$MODE" in
    compiler)
        SOURCE_FILE="${1:-examples/basic.peps}"
        sh "$ROOT_DIR/scripts/compiler/build.sh"
        exec "$ROOT_DIR/dist/compiler/linux.sh" "$SOURCE_FILE"
        ;;
    ide)
        sh "$ROOT_DIR/scripts/ide/build.sh"
        exec "$ROOT_DIR/dist/ide/linux.sh" "$@"
        ;;
    all)
        SOURCE_FILE="${1:-examples/basic.peps}"
        if [ "$#" -gt 0 ]; then
            shift
        fi

        sh "$ROOT_DIR/scripts/compiler/build.sh"
        "$ROOT_DIR/dist/compiler/linux.sh" "$SOURCE_FILE"

        sh "$ROOT_DIR/scripts/ide/build.sh"
        exec "$ROOT_DIR/dist/ide/linux.sh" "$@"
        ;;
    *)
        echo "Usage: sh scripts/build-run.sh [compiler [file.peps] | ide | all [file.peps]]" >&2
        exit 2
        ;;
esac
