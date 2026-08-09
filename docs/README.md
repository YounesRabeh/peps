# Development, Testing, and Releases

This guide explains how to run Peps locally, validate changes, produce the
distribution artifacts, and prepare a release.

## Repository layout

| Path | Purpose |
| --- | --- |
| `src/` | Rust lexer, parser, semantic checker, compiler, VM, CLI, and IDE server |
| `tests/` | Rust integration tests for every language layer |
| `ide/` | React/Vite browser IDE and its tests |
| `docs/` | Learning guides shown in the README and IDE |
| [`examples/basic/`](../examples/basic/) | Runnable Peps programs used by the learning guides |
| [`examples/algorithms/`](../examples/algorithms/) | Five complete, well-known algorithm implementations |
| `scripts/` | Build and packaging entry points |
| `dist/` | Generated release artifacts; rebuild instead of editing these files |

## Prerequisites

Install these before developing locally:

- Rust and Cargo (stable toolchain)
- Node.js and pnpm for the browser IDE

For a Linux-to-Windows cross-build, also install the MinGW compiler and Rust's
GNU Windows target:

```sh
# Fedora
sudo dnf install mingw64-gcc

# Ubuntu / Debian
sudo apt install gcc-mingw-w64-x86-64

rustup target add x86_64-pc-windows-gnu
```

Linux packaging uses `appimagetool`. The Linux build scripts use an installed
copy when available, otherwise download it to `.tools/` with `curl`.

## Run Peps locally

Run an example without producing a release package:

```sh
cargo run -- examples/basic/01-variables.peps
```

Build just the release CLI binary:

```sh
cargo build --release --bin peps
./target/release/peps examples/basic/01-variables.peps
```

Install the Rust WebAssembly target once. With rustup:

```sh
rustup target add wasm32-unknown-unknown
```

On Fedora when Rust is installed from the system packages:

```sh
sudo dnf install rust-std-static-wasm32-unknown-unknown
```

Then develop the browser IDE from the `ide` directory:

```sh
cd ide
pnpm install --frozen-lockfile
pnpm dev
```

`pnpm dev` builds the compiler as WebAssembly and starts Vite. Open the URL it
prints (normally `http://127.0.0.1:5173`). Programs execute entirely in the
browser and are not sent to a backend. Press `Ctrl+C` to stop Vite.

## GitHub Pages IDE

The Pages workflow in [`.github/workflows/pages.yml`](../.github/workflows/pages.yml)
tests and builds the browser IDE on pushes to `main` or `dev`, then deploys it
to [`https://younesrabeh.github.io/peps/`](https://younesrabeh.github.io/peps/).

To enable the first deployment, open the repository's GitHub settings, choose
**Pages**, and set **Source** to **GitHub Actions**. The deployment contains only
static HTML, CSS, JavaScript, and WebAssembly. It has no server, database, or
compiler API, and user programs remain on the user's device.

## Test before building artifacts

Run these commands from the repository root. They are the recommended release
gate because they cover Rust behavior, IDE behavior, formatting, linting, and
the production web bundle.

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings

cd ide
pnpm test -- --run
pnpm run build
```

Also run every numbered learning example. This catches documentation examples
that no longer match the language:

```sh
for example_file in examples/basic/*.peps examples/algorithms/*.peps; do
  cargo run --quiet -- "$example_file" || exit 1
done
```

On Windows PowerShell, use the same Rust commands and replace the IDE section
with:

```powershell
Set-Location ide
pnpm test -- --run
pnpm run build
Set-Location ..
```

## Build artifacts

The package commands recreate their output directory. Do not place manual files
inside `dist/compiler/<platform>` or `dist/ide/<platform>` because the next
build removes and replaces that directory.

### Linux packages

On Linux, build both packages with:

```sh
sh scripts/build-run.sh all
```

Or build one package at a time:

```sh
sh scripts/build-run.sh compiler
sh scripts/build-run.sh ide
```

This creates:

| Artifact | Purpose |
| --- | --- |
| `dist/compiler/linux/peps-<version>` | Standalone command-line compiler/runtime |
| `dist/compiler/linux/peps-bytecode-<version>` | Secondary CLI copy included by the compiler package |
| `dist/compiler/linux/linux-<version>.sh` | CLI launcher script |
| `dist/compiler/linux/peps-compiler-<version>-x86_64.AppImage` | Portable Linux compiler app |
| `dist/ide/linux/peps-ide-<version>-x86_64.AppImage` | Portable Linux IDE app |

`<version>` comes from the `version` field in `Cargo.toml`. For example,
version `0.8.1` produces `peps-compiler-0.8.1-x86_64.AppImage`.

Verify the CLI artifact before release:

```sh
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
./dist/compiler/linux/linux-$VERSION.sh examples/basic/01-variables.peps
```

From the repository root, start the IDE AppImage and confirm that it opens at
`http://127.0.0.1:5179`:

```sh
sh scripts/ide/build.sh
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
./dist/ide/linux/peps-ide-$VERSION-x86_64.AppImage
```

### Native Windows packages

Run this in PowerShell on Windows:

```powershell
.\scripts\build-run.ps1 all
```

The default target is `x86_64-pc-windows-msvc`. The build produces:

| Artifact | Purpose |
| --- | --- |
| `dist\compiler\windows\peps-<version>.exe` | Command-line compiler/runtime |
| `dist\compiler\windows\peps-<version>.cmd` | CLI launcher |
| `dist\ide\windows\peps-ide-<version>.exe` | IDE server executable |
| `dist\ide\windows\peps-ide-<version>.cmd` | IDE launcher |
| `dist\ide\windows\frontend\dist\` | Browser files required by the IDE executable |

From the repository root, verify both launchers:

```powershell
$Version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
.\dist\compiler\windows\peps-$Version.cmd examples\basic\01-variables.peps
.\dist\ide\windows\peps-ide-$Version.cmd
```

### Windows packages from Linux

With the MinGW and Rust prerequisites installed, run:

```sh
sh scripts/build-windows.sh
```

This produces the same Windows layout under `dist/compiler/windows/` and
`dist/ide/windows/`. The script builds the frontend with a locked pnpm install,
then copies the required `frontend/dist` directory beside the versioned
`peps-ide-<version>.exe`.

### macOS status

There is currently no local `.app`/DMG packaging script. On a Mac, you can
build local binaries with `cargo build --release --bin peps --bin peps-ide`
and build the frontend with `cd ide && pnpm run build`. The automated release
workflow packages the compiler and IDE as Intel macOS `.tar.gz` archives.

## Automated draft releases

[`.github/workflows/release.yml`](../.github/workflows/release.yml) runs when a
tag beginning with `v` is pushed. It can also be started manually from GitHub
Actions, in which case it selects the newest version tag.

The tag must match the package version in `Cargo.toml`. Read that version, then
create and push the matching tag:

```sh
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
git tag "v$VERSION"
git push origin "v$VERSION"
```

The workflow runs all Rust and IDE checks, builds compiler and IDE packages for
Linux x86_64, Windows x86_64, and Intel macOS, generates `SHA256SUMS`, then
creates a draft GitHub release. Review the generated notes and artifacts before
publishing the draft. The workflow uses the repository-provided `GITHUB_TOKEN`;
no release secret is required.

## Release checklist

1. Choose the version and update `version` in `Cargo.toml`; run `cargo check`
   so `Cargo.lock` is updated consistently.
2. Confirm `git status` contains only intentional changes.
3. Run every command in [Test before building artifacts](#test-before-building-artifacts).
4. Push the matching `v<version>` tag and let the release workflow build each
   platform artifact, or build them locally using the commands above.
5. Verify the CLI launcher and IDE launcher for every platform being released.
6. Create archives for multi-file Windows packages. The versioned
   `peps-ide-<version>.exe` must be distributed with its matching `.cmd` file
   and `frontend/dist/`; do not upload the EXE by itself.
7. Review the draft GitHub release and its generated release notes.
8. Verify the included `SHA256SUMS`, then publish the draft release.

On Windows, archive the complete package directories with PowerShell:

```powershell
$Version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
Compress-Archive -Path dist\compiler\windows\* -DestinationPath "dist\peps-compiler-$Version-windows-x86_64.zip" -Force
Compress-Archive -Path dist\ide\windows\* -DestinationPath "dist\peps-ide-$Version-windows-x86_64.zip" -Force
```

For Linux, the two AppImages can be uploaded directly. If you also distribute
the raw CLI, archive `peps-<version>` together with `linux-<version>.sh` so users retain the
launcher expected by the package.
