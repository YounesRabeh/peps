# 14. Map Key Existence

Put `🔑` before a map and a text key to check whether that key exists. The
expression returns `✅` when the key is present and `❌` when it is absent.

```peps
📖 🟰 🗺️ 💬name💬 ➡️ 💬Peps💬 💬year💬 ➡️ 💬2026💬 🗺️

📢 🔑 📖 💬name💬    // ✅
📢 🔑 📖 💬missing💬 // ❌
```

The key may also come from a text variable:

```peps
📝 🟰 💬year💬
🤔 🔑 📖 📝 🔓
    📢 📖 🔎 📝
🔒
```

Use `🔑` before `🔎` when a key may be absent. `🔑` never raises a missing-key
diagnostic, but its first operand must be a map and its second operand must be
text. Maps still support values of any one homogeneous type; key existence does
not inspect or change those values.

Try [the map key existence example](../examples/basic/14-map-key-existence.peps).

___
Back to: [Constants](13-constants.md).
