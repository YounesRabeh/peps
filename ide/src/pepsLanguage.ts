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
        // String delimiter first
        [/💬/, { token: "string.quote", next: "@string" }],

        // Longest operators first
        [/🟰🟰|❌🟰|◀️🟰|▶️🟰/, "operator"],

        // Emoji numbers before generic variables
        [/(?:0️⃣|1️⃣|2️⃣|3️⃣|4️⃣|5️⃣|6️⃣|7️⃣|8️⃣|9️⃣)+/, "number"],

        // Keywords
        [/📢|🤔|😐|🔁|🛑|⏭️|⏭|✅|❌/, "keyword"],

        // Single operators
        [/🟰|➕|➖|✖️|➗|◀️|▶️|🤝|🔀|🚫/, "operator"],

        // Delimiters
        [/🔓|🔒|🔚|📚/, "delimiter"],

        // Whitespace
        [/\s+/, "white"],

        // Safe fallback.
        // Keep this last. It avoids Monaco crashing on Unicode range regexes.
        [/[^\s]/u, "variable"]
      ],

      string: [
        [/💬/, { token: "string.quote", next: "@pop" }],
        [/[^💬]+/, "string"]
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
      { token: "string.quote", foreground: "fca5a5" },
      { token: "variable", foreground: "f8fafc" },
      { token: "invalid", foreground: "f87171" }
    ],
    colors: {
      "editor.background": "#151719",
      "editor.foreground": "#f8fafc",
      "editorLineNumber.foreground": "#64748b",
      "editorCursor.foreground": "#f8fafc",
      "editor.selectionBackground": "#334155"
    }
  });
  monaco.languages.registerCompletionItemProvider("peps", {
    triggerCharacters: [":"],
    provideCompletionItems(model, position) {
      return provideEmojiCompletionItems(monaco, model, position);
    }
  });
}
