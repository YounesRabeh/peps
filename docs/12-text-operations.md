# 12. Text operations

Use `📏` to count characters in text and `🔎` with a zero-based integer index
to read one character as text.

```peps
📝 🟰 💬Peps 🚀💬

📢 📏 📝
📢 📝 🔎 0️⃣
📢 📝 🔎 5️⃣
```

This prints `6`, `P`, and `🚀`. The operators also work directly with text
literals:

```peps
📢 📏 💬hello💬
📢 💬hello💬 🔎 1️⃣
```

Peps counts Unicode grapheme characters rather than bytes or Unicode code
points. A composed emoji such as `👨‍👩‍👧‍👦` therefore counts as one character
and is returned intact when indexed.

Indexes begin at zero and must be integers. Negative indexes and indexes at or
beyond the text length stop execution with an out-of-bounds runtime diagnostic.
The original text is not changed by indexing.

Try [the text-operations example](../examples/basic/12-text-operations.peps).
___
Back to: [Maps](11-maps.md).
