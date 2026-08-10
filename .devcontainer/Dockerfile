FROM mcr.microsoft.com/devcontainers/javascript-node:24-bookworm

ARG RUST_VERSION=1.97.1
ARG PNPM_VERSION=11.18.0
ARG PEPS_VERSION=dev

LABEL org.opencontainers.image.title="Peps development container" \
      org.opencontainers.image.description="Rust, WebAssembly, Node.js, pnpm, and Windows cross-build tools for Peps" \
      org.opencontainers.image.source="https://github.com/YounesRabeh/peps" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${PEPS_VERSION}"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        file \
        gcc-mingw-w64-x86-64 \
        git \
        libfuse2 \
        pkg-config \
    && npm install --global "pnpm@${PNPM_VERSION}" \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER node

ENV RUSTUP_HOME=/home/node/.rustup \
    CARGO_HOME=/home/node/.cargo \
    PATH=/home/node/.cargo/bin:${PATH}

RUN curl --fail --silent --show-error --location \
        https://sh.rustup.rs --output /tmp/rustup-init.sh \
    && sh /tmp/rustup-init.sh \
        -y \
        --no-modify-path \
        --profile minimal \
        --default-toolchain none \
    && rm /tmp/rustup-init.sh \
    && rustup toolchain install "${RUST_VERSION}" --profile minimal \
    && rustup default "${RUST_VERSION}" \
    && rustup component add clippy rustfmt \
    && rustup target add wasm32-unknown-unknown x86_64-pc-windows-gnu

USER root

RUN printf '%s\n' \
        'export RUSTUP_HOME=/home/node/.rustup' \
        'export CARGO_HOME=/home/node/.cargo' \
        'export PATH=/home/node/.cargo/bin:$PATH' \
        > /etc/profile.d/peps-rust.sh

USER node

WORKDIR /workspaces/peps
