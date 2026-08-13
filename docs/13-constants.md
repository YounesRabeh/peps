# 13. Constants

Put `🔐` before a one-emoji name to declare a read-only binding.

```peps
🔐 🐶 🟰 4️⃣2️⃣
🔐 📝 🟰 💬Peps💬

📢 🐶
📢 📝
```

Constants infer their type from the initializer and can hold any supported
value, including lists and maps. They can be read wherever an ordinary variable
can be read, but they cannot be reassigned:

```peps
🔐 🐶 🟰 4️⃣2️⃣
🐶 🟰 4️⃣3️⃣ // compile-time error
```

Collection constants are also protected from `📥`, because appending or merging
would replace their stored value:

```peps
🔐 🍎 🟰 📚 1️⃣ 2️⃣ 📚
🍎 📥 3️⃣ // compile-time error
```

Constants follow lexical scope. A constant declared inside a block or function
is available to nested blocks and disappears when its scope ends. Functions may
read top-level constants but cannot update them. A constant cannot reuse any
visible variable or constant name, preserving Peps's no-shadowing rule.

Like a fresh variable declaration, a constant's initializer is evaluated before
the new binding exists.

Try [the constants example](../examples/basic/13-constants.peps).
___
Back to: [Text operations](12-text-operations.md).

Next: [Map key existence](14-map-key-existence.md).
