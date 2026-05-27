![Peps banner](.github/PEPS-logo.png)

# Peps

Peps is an emoji-first programming language. This project includes a compiler and
a small bytecode runner for local `.peps` files.

## Syntax Reference

| Emoji | Name | Meaning | Example |
| --- | --- | --- | --- |
| 📢 | print | Print an expression | `📢 🐶 🔚` |
| 🤔 | if | Start an if statement | `🤔 🐶 🔓` |
| 😐 | else | Start an else block | `🔒 😐 🔓` |
| 🔁 | loop | Start a while loop or for loop | `🔁 🐶 🔓`, `🔁 🐾 🧭 🍎 🔓` |
| 🧭 | in | Separates for-loop item from source | `🔁 🐾 🧭 🍎 🔓` |
| 🔢 | range | Starts a numeric range source | `🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓` |
| ✅ | true | Boolean true | `🐶 🟰 ✅ 🔚` |
| ❌ | false / not-equal prefix | Boolean false, or part of `❌🟰` | `🐶 🟰 ❌ 🔚` |
| 🟰 | assign | Assign once / declare variable | `🐶 🟰 5️⃣ 🔚` |
| ➕ | add | Numeric addition | `1️⃣ ➕ 2️⃣` |
| ➖ | subtract / negate | Numeric subtraction or negative number | `5️⃣ ➖ 2️⃣`, `➖5️⃣` |
| ✖️ | multiply | Numeric multiplication | `2️⃣ ✖️ 3️⃣` |
| ➗ | divide | Numeric division | `6️⃣ ➗ 2️⃣` |
| 🟰🟰 | equals | Equality comparison | `🐶 🟰🟰 5️⃣` |
| ❌🟰 | not equals | Inequality comparison | `🐶 ❌🟰 5️⃣` |
| ◀️ | less than | Numeric less-than comparison | `2️⃣ ◀️ 3️⃣` |
| ▶️ | greater than | Numeric greater-than comparison | `5️⃣ ▶️ 3️⃣` |
| ◀️🟰 | less or equal | Numeric less-than-or-equal comparison | `2️⃣ ◀️🟰 3️⃣` |
| ▶️🟰 | greater or equal | Numeric greater-than-or-equal comparison | `5️⃣ ▶️🟰 3️⃣` |
| ➡️ | range end | Separates range start and end | `🔢 0️⃣ ➡️ 3️⃣` |
| 🔓 | block start | Open an if/else/while block | `🤔 ✅ 🔓` |
| 🔒 | block end | Close a block | `🔒` |
| 🔚 | statement end | End an assignment or print statement | `📢 🐶 🔚` |
| 💬 | string delimiter | Wrap string content | `🐶 🟰 💬hello💬 🔚` |
| 📚 | list delimiter | Start and end a list | `📚 1️⃣ 2️⃣ 📚` |
| 0️⃣ ... 9️⃣ | digits | Whole-number digits | `1️⃣2️⃣3️⃣` |
| any other single emoji | variable or emoji literal | Variable name, or literal on assignment RHS | `📦 🟰 🥊 🔚` |

## Loop Examples

For each item in a list:

```peps
🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚 🔚

🔁 🐾 🧭 🍎 🔓
    📢 🐾 🔚
🔒
```

For each number in a range. The end is exclusive, so this prints `0`, `1`, `2`:

```peps
🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓
    📢 🐾 🔚
🔒
```

While loops still use `🔁 condition`:

```peps
🌙 🟰 ❌ 🔚

🔁 🌙 🔓
    📢 🌙 🔚
🔒
```

## Running Peps Files

Peps source files use the `.peps` extension. The runner compiles the file to
bytecode, runs it, and prints the program output.

Create a file, for example `hello.peps`:

```peps
🐶 🟰 💬hello peps💬 🔚
📢 🐶 🔚
```

If you are working inside this Rust project, run a file with Cargo:

```sh
cargo run -- hello.peps
```

Run the included example the same way:

```sh
cargo run -- examples/basic.peps
```

After building the standalone binary, run a file without Cargo:

```sh
./dist/peps hello.peps
```

If `peps` is installed on your `PATH`, run files like this:

```sh
peps hello.peps
```

Compiler or runtime errors are printed with diagnostics. Valid programs print
only the program output.

## Build a Standalone `peps` Command

On Linux or macOS:

```sh
sh scripts/build-standalone.sh
./dist/peps examples/basic.peps
```

To run it as `peps basic.peps` from any folder, copy the binary into a folder on
your `PATH`:

```sh
mkdir -p "$HOME/.local/bin"
cp dist/peps "$HOME/.local/bin/peps"
peps examples/basic.peps
```

On Windows PowerShell:

```powershell
.\scripts\build-standalone.ps1
.\dist\peps.exe examples\basic.peps
```

The machine that builds the binary needs Rust. The machine that runs the copied
`dist/peps` or `dist/peps.exe` binary does not need Rust installed.
