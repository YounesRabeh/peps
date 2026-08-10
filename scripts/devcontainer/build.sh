#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
IMAGE_NAME="${PEPS_DEV_IMAGE:-ghcr.io/younesrabeh/peps-dev}"
PUSH_IMAGE=false

if [ "${1:-}" = "--push" ]; then
    PUSH_IMAGE=true
elif [ "$#" -ne 0 ]; then
    echo "usage: sh scripts/devcontainer/build.sh [--push]" >&2
    exit 2
fi

VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | sed -n '1p')
if [ -z "$VERSION" ]; then
    echo "error: could not read package version from Cargo.toml" >&2
    exit 1
fi

docker build \
    --file "$ROOT_DIR/.devcontainer/Dockerfile" \
    --build-arg "PEPS_VERSION=$VERSION" \
    --tag "$IMAGE_NAME:$VERSION" \
    --tag "$IMAGE_NAME:latest" \
    "$ROOT_DIR"

if [ "$PUSH_IMAGE" = true ]; then
    docker push "$IMAGE_NAME:$VERSION"
    docker push "$IMAGE_NAME:latest"
fi

echo "Built $IMAGE_NAME:$VERSION"
