# 10. Type conversion

Use `🔄` followed by a numeric type marker to convert a value explicitly.

| Syntax | Accepted value | Result |
| --- | --- | --- |
| `🔄 🔢 text` | Text containing an integer | Arbitrary-precision integer |
| `🔄 🔣 text` | Text containing a decimal number | 64-bit float |
| `🔄 🔣 integer` | Arbitrary-precision integer | 64-bit float |

```peps
📝 🟰 💬42💬
🐶 🟰 🔄 🔢 📝
🦊 🟰 🔄 🔣 🐶
🐱 🟰 🔄 🔣 💬3.5💬

📢 🐶 ➕ 1️⃣
📢 🦊
📢 🐱 ➕ 0️⃣.5️⃣
```

Conversion is an expression, so its result can be assigned, printed, returned,
or used in another expression. Leading and trailing whitespace in converted
text is ignored. A sign and ordinary keyboard digits are accepted inside text,
for example `💬-42💬` and `💬3.5💬`.

Invalid text stops execution with a runtime diagnostic. Explicit integer-to-
float conversion may round integers that a 64-bit float cannot represent
exactly, and rejects integers too large to produce a finite float. Implicit
mixed-number operations continue to reject lossy integer promotion.

Try [the type-conversion example](../examples/basic/10-type-conversion.peps).
___
Back to: [Input](09-input.md).

Next: [Maps](11-maps.md).
