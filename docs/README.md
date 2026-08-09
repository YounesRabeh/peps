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

To run the browser IDE during development:

```sh
cd ide
pnpm dev
```

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
| `dist/compiler/linux/peps` | Standalone command-line compiler/runtime |
| `dist/compiler/linux/peps-bytecode` | Secondary CLI copy included by the compiler package |
| `dist/compiler/linux/linux.sh` | CLI launcher script |
| `dist/compiler/linux/peps-compiler-x86_64.AppImage` | Portable Linux compiler app |
| `dist/ide/linux/peps-ide-x86_64.AppImage` | Portable Linux IDE app |

Verify the CLI artifact before release:

```sh
./dist/compiler/linux/linux.sh examples/basic/01-variables.peps
```

Start the IDE AppImage and confirm that it opens at `http://127.0.0.1:5179`:

```sh
./dist/ide/linux/peps-ide-x86_64.AppImage
```

### Native Windows packages

Run this in PowerShell on Windows:

```powershell
.\scripts\build-run.ps1 all
```

The default target is `x86_64-pc-windows-msvc`. The build produces:

| Artifact | Purpose |
| --- | --- |
| `dist\compiler\windows\peps.exe` | Command-line compiler/runtime |
| `dist\compiler\windows\peps.cmd` | CLI launcher |
| `dist\ide\windows\peps-ide.exe` | IDE server executable |
| `dist\ide\windows\peps-ide.cmd` | IDE launcher |
| `dist\ide\windows\frontend\dist\` | Browser files required by the IDE executable |

Verify both launchers:

```powershell
.\dist\compiler\windows\peps.cmd examples\01-variables.peps
.\dist\ide\windows\peps-ide.cmd
```

### Windows packages from Linux

With the MinGW and Rust prerequisites installed, run:

```sh
sh scripts/build-windows.sh
```

This produces the same Windows layout under `dist/compiler/windows/` and
`dist/ide/windows/`. The script builds the frontend with a locked pnpm install,
then copies the required `frontend/dist` directory beside `peps-ide.exe`.

### macOS status

There is currently no macOS packaging script or `.app`/DMG artifact. On a Mac,
you can build local binaries with `cargo build --release --bin peps --bin
peps-ide` and build the frontend with `cd ide && pnpm run build`. Add a macOS
packaging workflow before advertising a macOS release artifact.

## Release checklist

1. Choose the version and update `version` in `Cargo.toml`; run `cargo check`
   so `Cargo.lock` is updated consistently.
2. Confirm `git status` contains only intentional changes.
3. Run every command in [Test before building artifacts](#test-before-building-artifacts).
4. Build Linux artifacts on Linux and native Windows artifacts on Windows, or
   use the Linux cross-build for Windows.
5. Verify the CLI launcher and IDE launcher for every platform being released.
6. Create archives for multi-file Windows packages. `peps-ide.exe` must be
   distributed with `peps-ide.cmd` and `frontend/dist/`; do not upload the EXE
   by itself.
7. Create the Git tag and GitHub release according to your project's release
   policy, then upload the verified artifacts.
8. Include checksums and platform/architecture names in the release notes.

On Windows, archive the complete package directories with PowerShell:

```powershell
Compress-Archive -Path dist\compiler\windows\* -DestinationPath dist\peps-compiler-windows-x86_64.zip -Force
Compress-Archive -Path dist\ide\windows\* -DestinationPath dist\peps-ide-windows-x86_64.zip -Force
```

For Linux, the two AppImages can be uploaded directly. If you also distribute
the raw CLI, archive `peps` together with `linux.sh` so users retain the
launcher expected by the package.
