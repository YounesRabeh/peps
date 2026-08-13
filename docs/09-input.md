# 9. Input

Use `⌨️` followed by a type marker to read one value from standard input or the
IDE terminal.

| Syntax | Value read |
| --- | --- |
| `⌨️ 🔤` | Text |
| `⌨️ 🔢` | Arbitrary-precision integer |
| `⌨️ 🔣` | 64-bit float |
| `⌨️ ☑️` | Boolean |

```peps
📝 🟰 ⌨️ 🔤
🐶 🟰 ⌨️ 🔢
🦊 🟰 ⌨️ 🔣
🐱 🟰 ⌨️ ☑️

📢 📝
📢 🐶 ➕ 1️⃣
📢 🦊
📢 🐱
```

The CLI prompts when execution reaches each input expression. In the browser
IDE, press **Run**, enter the requested value in the terminal, and press
**Send**. The terminal preserves previous entries while the program requests
additional values.

Terminal integer and float values use ordinary keyboard digits, such as `42`
and `3.5`; emoji digits are only required in Peps source code. Boolean input
accepts `✅`, `❌`, `true`, or `false`. Text input preserves spaces.

Invalid input stops the program with a runtime diagnostic. Input inside loops
and functions reads a new terminal line each time the expression executes.

Try [the complete input example](../examples/basic/09-input.peps).
___
Back to: [Execution model](08-execution-model.md).

Next: [Type conversion](10-type-conversion.md).
