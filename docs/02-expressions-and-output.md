# 2. Expressions and output

Use `📢` to print a value. Numbers are written with keycap emoji digits and
have arbitrary precision: `1️⃣2️⃣3️⃣` means `123`.

```peps
📢 2️⃣ ➕ 3️⃣ ✖️ 4️⃣
📢 8️⃣ ➖ 3️⃣
📢 8️⃣ ➗ 2️⃣
```

Comparisons produce `✅` or `❌`. Combine boolean values with `🤝` (and), `🔀`
(or), and `🚫` (not).

```peps
📢 5️⃣ ▶️ 3️⃣
📢 🚫 ❌ 🤝 ✅
```

Text is delimited by `💬` and can be concatenated with `➕`.

```peps
📝 🟰 💬hello💬 ➕ 💬 peps💬
📢 📝
```

Raw text literals are used while assigning or concatenating text; print the
variable that holds the resulting text rather than printing a raw literal.

Try [the complete example](../examples/basic/02-expressions-and-output.peps).
___
Next: [Conditionals](03-conditionals.md).
