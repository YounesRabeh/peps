![Peps banner](.github/PEPS-logo.png)
Peps is an emoji-first programming language with:
- a Rust compiler/runtime for `.peps` files
- a local browser IDE powered by the same runtime

## Language Rules (Current)

1. Statements are separated by **new lines**.
2. `🔚` is optional and not required.
3. Variable identifiers must be **exactly one emoji**.
4. `break` (`🛑`) and `continue` (`⏭️`) are valid only inside loops.
5. `//` starts a line comment that runs to the end of the line.
6. Logical operators use `🤝`, `🔀`, and `🚫`.
7. Loop blocks use this structure:

```peps
🔁 ✅ 🔓
    ⏭️
    🛑
🔒
```
8. Assignments update the nearest visible variable when its type matches; otherwise they declare a variable in the current scope.
9. Variables declared inside `if`, `else`, `while`, or `for` blocks are visible only in that block and its nested blocks.
10. `for` iterator names are block-local and cannot reuse the name of an outer variable.

## Core Syntax

| Emoji | Meaning | Example |
| --- | --- | --- |
| 📢 | print | `📢 🐶` |
| 🤔 | if | `🤔 ✅ 🔓` |
| 😐 | else | `🔒 😐 🔓` |
| 🔁 | while / for . in . | `🔁 ✅ 🔓`, `🔁 🐾 🧭 🍎 🔓` |
| 🛑 | break | `🛑` |
| ⏭️ | continue | `⏭️` |
| 🧭 | in (for loops) | `🔁 🐾 🧭 🍎 🔓` |
| 🔢 | range | `🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓` |
| 🟰 | assign | `🐶 🟰 5️⃣` |
| ➕ | add / text concat | `1️⃣ ➕ 2️⃣`, `💬hi💬 ➕ 💬there💬` |
| ➖ | subtract | `5️⃣ ➖ 2️⃣` |
| ✖️ | multiply | `2️⃣ ✖️ 3️⃣` |
| ➗ | divide | `6️⃣ ➗ 2️⃣` |
| ▶️ / ◀️ | greater / less than | `5️⃣ ▶️ 3️⃣`, `2️⃣ ◀️ 4️⃣` |
| ▶️🟰 / ◀️🟰 | greater-or-equal / less-or-equal | `5️⃣ ▶️🟰 5️⃣` |
| 🟰🟰 / ❌🟰 | equal / not equal | `✅ 🟰🟰 ✅` |
| 📏 | list length | `📏 🍎` |
| 🔎 | list index | `🍎 🔎 1️⃣` |
| 📥 | list append / extend | `🍎 📥 4️⃣`, `🍎 📥 4️⃣ 5️⃣`, `🥝 🟰 🍎 📥 6️⃣ 5️⃣` |
| 🔓 / 🔒 | block start / end | `🤔 ✅ 🔓 ... 🔒` |
| 💬 | string delimiter | `🐶 🟰 💬hello💬` |
| 📚 | list delimiter | `🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚` |
| `//` | line comment | `📢 1️⃣ // ignored` |
| `🤝` / `🔀` / `🚫` | logical operators (AND/OR/NOT) | `✅ 🤝 ❌` |

## Variables and Scope

`🟰` declares a variable when its emoji name is not already visible. Reusing a
visible name updates that variable, but the new value must have the same type:

```peps
🐶 🟰 1️⃣
🤔 ✅ 🔓
    🐶 🟰 🐶 ➕ 1️⃣ // updates the outer variable
    🐱 🟰 5️⃣       // local to this block
    📢 🐱
🔒
📢 🐶
📢 🐱 // 🐱 is an emoji literal here, not the expired local variable
```

Each `if` branch and each loop body has its own lexical scope. Local variables
are available to nested blocks. A `for` iterator is local to its loop, and its
name must not conflict with another visible variable.

## Example

```peps
🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚
🔁 🐾 🧭 🍎 🔓
    📢 🐾
🔒
```

## Run a `.peps` File

```sh
cargo run -- examples/basic.peps
```

## Test

```sh
cargo test
cd ide && npm test
```

## Start IDE

```sh
sh scripts/ide/build.sh
./dist/ide/linux/peps-ide-x86_64.AppImage
```

Then open: `http://127.0.0.1:5179`

## One-command Helpers

Linux/macOS:

```sh
sh scripts/build-run.sh compiler
sh scripts/build-run.sh ide
sh scripts/build-run.sh all
```

Linux build artifacts are written to:
- `dist/compiler/linux/linux.sh`
- `dist/compiler/linux/peps!`
- `dist/compiler/linux/peps!-bytecode`
- `dist/compiler/linux/peps-compiler-x86_64.AppImage`
- `dist/ide/linux/peps-ide-x86_64.AppImage`

Windows PowerShell:

```powershell
.\scripts\build-run.ps1 compiler
.\scripts\build-run.ps1 ide
.\scripts\build-run.ps1 all
```

Windows build artifacts are written to:
- `dist\compiler\windows\peps!.exe`
- `dist\ide\windows\peps-ide.exe`

Cross-build Windows `.exe` files from Linux:

```sh
sh scripts/build-windows.sh
```

Requires:
- `x86_64-w64-mingw32-gcc`
- Rust target `x86_64-pc-windows-gnu`

### IDE Preview
![Peps IDE](.github/peps-ide.png)
