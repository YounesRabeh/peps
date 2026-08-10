# Peps development container

The development container provides a reproducible Peps environment for VS Code
Dev Containers and GitHub Codespaces. It includes Rust, `rustfmt`, Clippy, the
WebAssembly and Windows GNU Rust targets, Node.js, pnpm, MinGW, and the Linux
packaging tools. After the container is created, it installs the locked IDE
dependencies and fetches Rust crates.

## Open the checked-in configuration

Install Docker and the VS Code **Dev Containers** extension, open the Peps
repository in VS Code, then run **Dev Containers: Reopen in Container** from
the Command Palette. GitHub Codespaces uses the same configuration.

The checked-in `devcontainer.json` builds `.devcontainer/Dockerfile` locally,
so this works even before an image is published to GitHub Container Registry.

Forwarded ports:

| Port | Purpose |
| --- | --- |
| `5173` | Browser IDE started with `pnpm --dir ide dev` |
| `5179` | Packaged Peps IDE server |

## Build the image locally

Run this from the repository root:

```sh
sh scripts/devcontainer/build.sh
```

The script reads the version from `Cargo.toml` and tags one local image as:

```text
ghcr.io/younesrabeh/peps-dev:<version>
ghcr.io/younesrabeh/peps-dev:latest
```

`latest` moves to the newest build; the version tag is the reproducible choice.

## Upload the image to GHCR

Create a GitHub **personal access token (classic)** with only
`write:packages`. Store it in a local secret file, then authenticate and upload
both tags from the repository root:

```sh
mkdir -p .secrets
printf '%s' 'github_pat_...' > .secrets/ghcr-token
chmod 600 .secrets/ghcr-token
```

> [!IMPORTANT]
>
> Upload the versioned image and `latest` tag with:
>
> ```sh
> docker login ghcr.io -u YounesRabeh --password-stdin < .secrets/ghcr-token
> sh scripts/devcontainer/build.sh --push
> ```

`.secrets/` is ignored by Git. Do not add the token to any tracked file,
including this guide, Dockerfile, or `devcontainer.json`. The token is also
stored by Docker after login; run `docker logout ghcr.io` when you no longer
need this machine to be authenticated.

The first manually published package is private by default. Change its
visibility in GitHub Packages if it should be publicly pullable. Release tags
also publish this image automatically through the release workflow using its
repository `GITHUB_TOKEN`.

## Use the published image

For a faster setup, replace the `build` object in `devcontainer.json` with a
specific published version:

```json
"image": "ghcr.io/younesrabeh/peps-dev:1.2.0"
```

Use the version that matches the release you want. A private package requires
Docker to be authenticated to `ghcr.io`; a public package can be pulled without
authentication. Prefer a version tag over `:latest` whenever reproducibility
matters.
