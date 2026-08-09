# 4. Loops

`🔁` creates a while loop when followed by a condition. Update a value inside
the body so the loop can finish.

```peps
🐶 🟰 0️⃣
🔁 🐶 ◀️ 3️⃣ 🔓
    📢 🐶
    🐶 🟰 🐶 ➕ 1️⃣
🔒
```

Use `🔁 <emoji> 🧭 <list>` to loop over a list, or use `🔢 start ➡️ end` for a
numeric range. The end of a range is exclusive.

```peps
🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓
    📢 🐾
🔒
```

`🛑` stops the nearest loop and `⏭️` skips to its next iteration. Iterator
names are local to the loop and cannot conflict with an outer binding. See the
[loop example](../examples/basic/04-loops.peps).
___
[Lists](05-lists.md).
