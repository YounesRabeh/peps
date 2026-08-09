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
8. Assignments update the nearest visible variable, including its type; otherwise they declare a variable in the current scope.
9. Variables declared inside `if`, `else`, `while`, or `for` blocks are visible only in that block and its nested blocks.
10. `for` iterator names are block-local and cannot reuse the name of an outer variable.
11. Numbers are arbitrary-precision integers and do not overflow at `i64` limits.
12. Compiler and CLI runs have no instruction limit; browser IDE runs stop after 100,000 instructions for safety.
13. Functions are named at top level with `🧩`, called with `📞`, and return a value with `↩️`.
14. Every function path must return. Functions can access globals, but cannot access caller-local variables.

## Core Syntax

| Emoji | Meaning | Example |
| --- | --- | --- |
| 📢 | print | `📢 🐶` |
| 🤔 | if | `🤔 ✅ 🔓` |
| 😐 | else | `🔒 😐 🔓` |
| 🔁 | while / for . in . | `🔁 ✅ 🔓`, `🔁 🐾 🧭 🍎 🔓` |
| 🛑 | break | `🛑` |
| ⏭️ | continue | `⏭️` |
| 🧩 | define function | `🧩 🧮 📚 🐶 🐱 📚 🔓 ... 🔒` |
| 📞 | call function | `📞 🧮 📚 1️⃣ 2️⃣ 📚` |
| ↩️ | return from function | `↩️ 🐶 ➕ 🐱` |
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
visible name updates that variable and may change its type:

```peps
🐶 🟰 1️⃣
🤔 ✅ 🔓
    🐶 🟰 ✅       // updates the outer variable and changes it to bool
    🐱 🟰 5️⃣       // local to this block
    📢 🐱
🔒
📢 🐶
📢 🐱 // 🐱 is an emoji literal here, not the expired local variable
```

Each `if` branch and each loop body has its own lexical scope. Local variables
are available to nested blocks. A `for` iterator is local to its loop, and its
name must not conflict with another visible variable.

## Functions

Functions are defined at the top level. Their names and parameter names must
each be exactly one emoji, and parameter names must be unique. Definitions are
collected before the rest of the program is checked, so calls may appear before
a definition and functions may call themselves recursively.

```peps
🧩 🧮 📚 🐶 🐱 📚 🔓
    ↩️ 🐶 ➕ 🐱
🔒

🐸 🟰 📞 🧮 📚 1️⃣ 2️⃣ 📚
📞 🧮 📚 3️⃣ 4️⃣ 📚 // result is discarded
```

Arguments are positional. Parameters and function results are dynamically
typed, so operations involving them are checked when the function runs. Known
type errors elsewhere remain compile-time errors. Every possible path through
a function must explicitly reach `↩️`; an `if` satisfies this rule only when
both branches return, and a loop alone does not.

Each call has isolated parameter and local-variable storage, including during
recursion. Functions may read and reassign top-level variables, but they cannot
see a caller's block locals. Parameters and function locals cannot shadow a
visible global. Function and variable names occupy separate namespaces, and
functions are not values.

## Execution Model

Peps numbers have arbitrary precision. Together with mutable variables,
conditionals, and unrestricted `while` loops, this allows Peps to simulate a
two-counter Minsky machine and makes the language Turing complete under the
usual idealized assumption of unbounded memory.

The compiler and command-line runner execute without an instruction limit, so
a non-terminating program will continue running until it is interrupted. The
browser IDE applies a 100,000-instruction limit so an accidental infinite loop
cannot hang its local server indefinitely.

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
cd ide && pnpm test
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
- `dist/compiler/linux/peps`
- `dist/compiler/linux/peps-bytecode`
- `dist/compiler/linux/peps-compiler-x86_64.AppImage`
- `dist/ide/linux/peps-ide-x86_64.AppImage`

Windows PowerShell:

```powershell
.\scripts\build-run.ps1 compiler
.\scripts\build-run.ps1 ide
.\scripts\build-run.ps1 all
```

Windows build artifacts are written to:
- `dist\compiler\windows\peps.exe`
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
