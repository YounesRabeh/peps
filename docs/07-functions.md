# 7. Functions

Functions are named at the top level with `🧩`. A function name and every
parameter name are exactly one emoji. Parameters are between `📚` delimiters;
return a value with `↩️`.

```peps
🧩 🧮 📚 🐶 🐱 📚 🔓
    ↩️ 🐶 ➕ 🐱
🔒

🐸 🟰 📞 🧮 📚 1️⃣ 2️⃣ 📚
📢 🐸
```

Calls use `📞` and positional arguments. A standalone call discards its return
value. Peps collects definitions before checking calls, so forward calls and
recursion work.

Every path through a function must return. An `if` only satisfies that rule
when both branches return; a loop alone does not. Calls have isolated local
storage, can read and update top-level variables, and cannot see a caller's
block-local variables.

Parameters and return values are dynamically typed, so an operation that
depends on either is checked when the function runs. Known values keep normal
compile-time type checks. Parameters and function-local variables cannot shadow
a visible global; function and variable names are separate namespaces, and
functions cannot be stored, passed, or returned as values.

Run [the function example](../examples/basic/07-functions.peps).
___
Next: [Execution model](08-execution-model.md).
