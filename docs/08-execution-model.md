# 8. Execution model

Statements are separated by newlines. `🔚` is an optional explicit statement
separator. `//` begins a line comment.

```peps
🐶 🟰 1️⃣ // this is a comment
🐶 🟰 🐶 ➕ 1️⃣ 🔚
📢 🐶
```

Integers have arbitrary precision, so they do not overflow at `i64` limits.
Floats use finite 64-bit IEEE 754 values; arithmetic rejects division by zero
and non-finite results. Together with mutable integer variables, conditionals,
and unrestricted while loops, Peps can simulate a two-counter Minsky machine
and is Turing complete under the usual unbounded-memory assumption.

The compiler and command-line runner have no instruction limit. The browser
IDE applies a 100,000-instruction safety limit, so stop or correct an accidental
infinite loop before it reaches that limit. Run the
[execution example](../examples/basic/08-execution-model.peps). For larger
working programs, explore the [algorithm examples](../examples/algorithms/).

Next: [Input](09-input.md).
