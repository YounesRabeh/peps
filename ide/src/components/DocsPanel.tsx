import { useState } from "react";
const variablesExample = `// Declare, update, and change a variable's type.
🐶 🟰 5️⃣
🐶 🟰 🐶 ➕ 2️⃣
📢 🐶

🐶 🟰 ✅
📢 🐶

// An unknown emoji is a literal value.
📢 🦊
`;

const expressionsExample = `🐶 🟰 2️⃣ ➕ 3️⃣ ✖️ 4️⃣
🐱 🟰 🐶 ▶️ 1️⃣0️⃣
📝 🟰 💬hello💬 ➕ 💬 peps💬

📢 🐶
📢 🐱
📢 🚫 ❌ 🤝 ✅
📢 📝
`;

const conditionalsExample = `🐶 🟰 4️⃣
🟢 🟰 💬large💬
🔴 🟰 💬small💬

🤔 🐶 ▶️ 3️⃣ 🔓
    📢 🟢
🔒 😐 🔓
    📢 🔴
🔒
`;

const loopsExample = `🐶 🟰 0️⃣
🔁 🐶 ◀️ 3️⃣ 🔓
    📢 🐶
    🐶 🟰 🐶 ➕ 1️⃣
🔒

🍎 🟰 📚 4️⃣ 5️⃣ 6️⃣ 📚
🔁 🐾 🧭 🍎 🔓
    🤔 🐾 🟰🟰 5️⃣ 🔓
        ⏭️
    🔒
    📢 🐾
🔒

🔁 🐾 🧭 🔢 0️⃣ ➡️ 3️⃣ 🔓
    📢 🐾
🔒
`;

const listsExample = `🍎 🟰 📚 1️⃣ 2️⃣ 3️⃣ 📚
🍎 📥 4️⃣
🍎 📥 5️⃣ 6️⃣

📢 🍎
📢 📏 🍎
📢 🍎 🔎 1️⃣
`;

const scopeExample = `🐶 🟰 1️⃣

🤔 ✅ 🔓
    🐶 🟰 🐶 ➕ 1️⃣
    🐱 🟰 5️⃣
    📢 🐱
🔒

📢 🐶
// 🐱 is now an emoji literal, not the block-local variable.
📢 🐱
`;

const functionsExample = `🧩 🧮 📚 🐶 🐱 📚 🔓
    ↩️ 🐶 ➕ 🐱
🔒

🧩 🌀 📚 🐶 📚 🔓
    🤔 🐶 ◀️🟰 1️⃣ 🔓
        ↩️ 1️⃣
    🔒 😐 🔓
        ↩️ 🐶 ✖️ 📞 🌀 📚 🐶 ➖ 1️⃣ 📚
    🔒
🔒

📢 📞 🧮 📚 1️⃣ 2️⃣ 📚
📢 📞 🌀 📚 5️⃣ 📚
`;

const executionExample = `// 🔚 is optional; comments run to the end of the line.
🐶 🟰 9️⃣2️⃣2️⃣3️⃣3️⃣7️⃣2️⃣0️⃣3️⃣6️⃣8️⃣5️⃣4️⃣7️⃣7️⃣5️⃣8️⃣0️⃣8️⃣
🐶 🟰 🐶 ➕ 1️⃣ 🔚
📢 🐶
`;

type DocsPanelProps = {
  onLoadExample: (source: string) => void;
};

type Guide = {
  number: number;
  title: string;
  description: string;
  points: string[];
  example: string;
};

const guides: Guide[] = [
  {
    number: 1,
    title: "Variables",
    description: "Use one emoji as a variable name and 🟰 to declare or update it.",
    points: [
      "A new visible name declares a variable.",
      "Reusing a visible name updates it and may change its type.",
      "An unknown emoji expression is an emoji literal."
    ],
    example: variablesExample
  },
  {
    number: 2,
    title: "Expressions and output",
    description: "Print values with 📢 and combine values with Peps operators.",
    points: [
      "Numbers use emoji digits and have arbitrary precision.",
      "Use ➕, ➖, ✖️, and ➗ for arithmetic.",
      "Use 🤝, 🔀, and 🚫 for boolean logic."
    ],
    example: expressionsExample
  },
  {
    number: 3,
    title: "Conditionals",
    description: "Use 🤔 with a boolean condition, and 😐 for the optional else branch.",
    points: [
      "A conditional body is enclosed by 🔓 and 🔒.",
      "Conditions must evaluate to ✅ or ❌.",
      "Each branch has its own scope."
    ],
    example: conditionalsExample
  },
  {
    number: 4,
    title: "Loops",
    description: "Use 🔁 for while loops, list iteration, and numeric ranges.",
    points: [
      "🔁 condition 🔓 ... 🔒 repeats while the condition is true.",
      "🔁 🐾 🧭 list iterates items; 🔢 start ➡️ end iterates a range.",
      "🛑 stops and ⏭️ skips an iteration."
    ],
    example: loopsExample
  },
  {
    number: 5,
    title: "Lists",
    description: "Create ordered, homogeneous lists between matching 📚 delimiters.",
    points: [
      "📏 gets a list length and 🔎 reads a zero-based item.",
      "📥 appends one value or extends with several values.",
      "All list items must have the same type."
    ],
    example: listsExample
  },
  {
    number: 6,
    title: "Scope",
    description: "Blocks create lexical scopes for fresh variables.",
    points: [
      "Nested blocks can read outer variables.",
      "Assignments update the nearest visible binding.",
      "Block locals become emoji literals after their block ends."
    ],
    example: scopeExample
  },
  {
    number: 7,
    title: "Functions",
    description: "Define top-level functions with 🧩, call them with 📞, and return with ↩️.",
    points: [
      "Arguments are positional and enclosed by 📚.",
      "Functions support forward calls and recursion.",
      "Every possible path through a function must return."
    ],
    example: functionsExample
  },
  {
    number: 8,
    title: "Execution model",
    description: "Newlines separate statements; 🔚 is an optional explicit separator.",
    points: [
      "// begins a line comment.",
      "CLI programs have no instruction limit.",
      "The browser IDE stops execution after 100,000 instructions for safety."
    ],
    example: executionExample
  }
];

export function DocsPanel({ onLoadExample }: DocsPanelProps) {
  const [selectedGuide, setSelectedGuide] = useState(0);
  const guide = guides[selectedGuide];

  return (
    <aside className="docs-panel" aria-label="Peps documentation">
      <div className="docs-header">
        <div>
          <h2>Docs</h2>
          <p>Learn Peps step by step.</p>
        </div>
        <span className="docs-progress">{guide.number}/8</span>
      </div>

      <nav className="docs-nav" aria-label="Documentation guides">
        {guides.map((item, index) => (
          <button
            className={index === selectedGuide ? "docs-nav-item active" : "docs-nav-item"}
            key={item.number}
            onClick={() => setSelectedGuide(index)}
            type="button"
          >
            <span>{item.number}</span>
            {item.title}
          </button>
        ))}
      </nav>

      <article className="docs-content">
        <h3>{guide.number}. {guide.title}</h3>
        <p>{guide.description}</p>
        <ul>
          {guide.points.map((point) => <li key={point}>{point}</li>)}
        </ul>
        <button className="load-example-button" onClick={() => onLoadExample(guide.example)} type="button">
          Load example into editor
        </button>
      </article>
    </aside>
  );
}
