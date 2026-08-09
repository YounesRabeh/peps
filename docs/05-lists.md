# 5. Lists

Create a homogeneous list with matching `📚` delimiters. List items keep their
source order.

```peps
🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚
📢 🍎
```

`📏` returns a list's length and `🔎` reads a zero-based index.

```peps
📢 📏 🍎
📢 🍎 🔎 1️⃣
```

Append one item or extend with several items using `📥`.

```peps
🍎 📥 4️⃣
🍎 📥 5️⃣ 6️⃣
```

Lists retain one element type. Run [the list example](../examples/basic/05-lists.peps).
___
Next: [Scope](06-scope.md).
