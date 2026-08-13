# 11. Maps

Maps associate text keys with values. Put key-value pairs between matching
`🗺️` delimiters and separate each key from its value with `➡️`.

```peps
📖 🟰 🗺️
    💬visits💬 ➡️ 1️⃣2️⃣0️⃣
    💬year💬 ➡️ 2️⃣0️⃣2️⃣6️⃣
🗺️

📢 📖 🔎 💬year💬
📢 📏 📖
```

Map keys must be text. Like lists, all values in one map must have the same
type. Values can be text, integers, floats, booleans, or emoji values; maps are
not limited to text-to-text pairs. Separate maps can use different value types.
An empty map is rejected because its value type cannot be inferred. `🔎` looks
up a key and `📏` returns the number of unique keys. Looking up a missing key
stops execution with a runtime diagnostic.

```peps
🚦 🟰 🗺️ 💬ready💬 ➡️ ✅ 💬cached💬 ➡️ ❌ 🗺️
📢 🚦 🔎 💬ready💬
```

Use `📥` with another map to insert and update entries:

```peps
📖 📥 🗺️
    💬visits💬 ➡️ 1️⃣3️⃣5️⃣
    💬users💬 ➡️ 4️⃣2️⃣
🗺️
```

Maps preserve insertion order when printed. A later duplicate key replaces the
old value without changing that key's position. Map literals, lookup, size, and
merge are expressions, so maps work with variables, blocks, and functions.

Try [the map example](../examples/basic/11-maps.peps).
___
Back to: [Type conversion](10-type-conversion.md).

Next: [Text operations](12-text-operations.md).
