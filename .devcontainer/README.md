# Peps development container

Open the repository in VS Code and choose **Dev Containers: Reopen in
Container**, or create a GitHub Codespace. The container installs the locked
pnpm dependencies and fetches Rust crates after it is created.

The checked-in configuration builds the image locally so it works before a
GHCR package exists. It includes OCI package metadata and can be published
later as a GitHub Package:

```sh
printf '%s' "$GHCR_TOKEN" | docker login ghcr.io --username YounesRabeh --password-stdin
sh scripts/devcontainer/build.sh --push
```

The build script reads `Cargo.toml` each time. It automatically sets the OCI
image version and creates both `ghcr.io/younesrabeh/peps-dev:<Cargo version>`
and `ghcr.io/younesrabeh/peps-dev:latest`; no version is copied into this
configuration.

After publishing, replace the `build` property in `devcontainer.json` with:

```json
"image": "ghcr.io/younesrabeh/peps-dev:<Cargo version>"
```

Use a GitHub token with `write:packages` for `GHCR_TOKEN`. A public package can
be pulled without authentication.
