# 2. Expressions and output

Use `📢` to print a value. Integers are written with keycap emoji digits and
have arbitrary precision: `1️⃣2️⃣3️⃣` means `123`. Put `.` between emoji digits
for a floating-point value: `1️⃣2️⃣.3️⃣4️⃣` means `12.34`.

```peps
📢 2️⃣ ➕ 3️⃣ ✖️ 4️⃣
📢 8️⃣ ➖ 3️⃣
📢 8️⃣ ➗ 2️⃣
📢 1️⃣.5️⃣ ➕ 2️⃣
📢 1️⃣.0️⃣ ➗ 3️⃣
📢 ➖5️⃣
📢 ➖1️⃣.5️⃣
```

Integer-only arithmetic keeps arbitrary precision, and integer-only division
uses integer division. If either operand is a float, Peps promotes the operation
to a 64-bit floating-point calculation. Promotion reports a runtime error when
an integer cannot be represented exactly as a float. Numeric ranges and list
indexes still require integers.

Put `➖` directly before an integer, float, variable, or numeric expression to
negate it. Negative values work in arithmetic, comparisons, lists, function
arguments, and integer ranges. Because list items have no comma, store adjacent
negative values in variables before placing them in the same list so `➖` is
not read as subtraction.

Comparisons produce `✅` or `❌`. Combine boolean values with `🤝` (and), `🔀`
(or), and `🚫` (not).

```peps
📢 5️⃣ ▶️ 3️⃣
📢 🚫 ❌ 🤝 ✅
```

Text is delimited by `💬` and can be concatenated with `➕`.

```peps
📝 🟰 💬hello💬 ➕ 💬 peps💬
📢 📝
```

Raw text literals are used while assigning or concatenating text; print the
variable that holds the resulting text rather than printing a raw literal.

Try [the complete example](../examples/basic/02-expressions-and-output.peps).
___
Next: [Conditionals](03-conditionals.md).
