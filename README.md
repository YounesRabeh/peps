![Peps banner](.github/PEPS-logo.png)

Peps is an emoji-first programming language with a Rust compiler/runtime for
`.peps` files and a local browser IDE powered by the same runtime.

## Get Started

Follow the guides in order. Every guide has a matching runnable program in
[`examples/basic/`](examples/basic/).

1. [Variables](docs/01-variables.md) 
2. [Expressions and output](docs/02-expressions-and-output.md)
3. [Conditionals](docs/03-conditionals.md) 
4. [Loops](docs/04-loops.md)
5. [Lists](docs/05-lists.md) 
6. [Scope](docs/06-scope.md) 
7. [Functions](docs/07-functions.md) 
8. [Execution model](docs/08-execution-model.md)

## Examples

- [Basic examples](examples/basic/) progress from variables to functions.
- [Algorithms](examples/algorithms/) contains five well-known algorithms
  recreated in Peps.

Run any example from the project root:

```sh
cargo run -- examples/basic/01-variables.peps
```

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

## Run, Test, and Build

Run the Rust and browser IDE checks before opening a pull request or making a
release:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings

cd ide
pnpm test -- --run
pnpm run build
```

Develop the browser IDE from the `ide` directory:

```sh
cd ide
pnpm dev
```

This starts both Vite and the Rust compiler API. Open the Vite address it
prints (normally `http://127.0.0.1:5173`); Run requests are forwarded to the
compiler server at `http://127.0.0.1:5179`. Press `Ctrl+C` to stop both.

For platform requirements, all artifact paths, Linux-to-Windows cross-builds,
verification, archives, and release uploads, see
[Development, Testing, and Releases](docs/README.md).

### IDE Preview

![Peps IDE](.github/peps-ide.png)
