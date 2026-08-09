# 1. Variables

Variables use one emoji as their name. Assign with `🟰`; the first assignment
declares the variable and a later assignment updates the nearest visible one.

```peps
🐶 🟰 5️⃣
🐶 🟰 🐶 ➕ 2️⃣
📢 🐶
```

Peps infers a value's type. Reassignment may intentionally change that type:

```peps
🐶 🟰 7️⃣
🐶 🟰 ✅
📢 🐶
```

An unbound emoji used as an expression is an emoji literal, so `📢 🦊` prints
`🦊`. See the [matching example](../examples/basic/01-variables.peps).
___
Next: [Expressions and output](02-expressions-and-output.md).
