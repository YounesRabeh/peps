import type * as Monaco from "monaco-editor";
import { provideEmojiCompletionItems } from "./emojiAutocomplete";

let registered = false;

export function registerPepsLanguage(monaco: typeof Monaco): void {
  if (registered) {
    return;
  }
  registered = true;

  monaco.languages.register({ id: "peps" });

  monaco.languages.setMonarchTokensProvider("peps", {
    tokenizer: {
      root: [
        [/💬/, { token: "string.quote", next: "@string" }],
        [/[📢🤔😐🔁✅❌🧭🔢]/u, "keyword"],
        [/(🟰🟰|❌🟰|◀️🟰|▶️🟰|🟰|➕|➖|✖️|➗|◀️|▶️|➡️)/u, "operator"],
        [/[🔓🔒🔚📚]/u, "delimiter"],
        [/(?:0️⃣|1️⃣|2️⃣|3️⃣|4️⃣|5️⃣|6️⃣|7️⃣|8️⃣|9️⃣)+/u, "number"],
        [/\S/u, "variable"]
      ],
      string: [
        [/💬/, { token: "string.quote", next: "@pop" }],
        [/[^💬]+/u, "string"]
      ]
    }
  });

  monaco.editor.defineTheme("peps-dark", {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "keyword", foreground: "ffcc66" },
      { token: "operator", foreground: "7dd3fc" },
      { token: "delimiter", foreground: "c4b5fd" },
      { token: "number", foreground: "86efac" },
      { token: "string", foreground: "fca5a5" },
      { token: "variable", foreground: "f8fafc" }
    ],
    colors: {
      "editor.background": "#101318"
    }
  });

  monaco.languages.registerCompletionItemProvider("peps", {
    triggerCharacters: [":"],
    provideCompletionItems(model, position) {
      return provideEmojiCompletionItems(monaco, model, position);
    }
  });
}
