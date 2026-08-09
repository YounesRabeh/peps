# 6. Scope

Every `if`, `else`, `while`, and `for` block has its own lexical scope. A block
can read bindings from outside it. Assigning to an already visible name updates
that binding; a fresh name is local to the current block.

```peps
🐶 🟰 1️⃣
🤔 ✅ 🔓
    🐶 🟰 🐶 ➕ 1️⃣
    🐱 🟰 5️⃣
    📢 🐱
🔒
📢 🐶
```

After the block, `🐶` is `2`, but `🐱` is no longer a variable. A later `🐱`
expression is interpreted as the literal emoji `🐱`. Sibling `if` and `else`
branches have independent scopes.

Explore [the scope example](../examples/basic/06-scope.peps).
___
[Functions](07-functions.md).
