import { describe, expect, it } from "vitest";
import {
  applyEmojiCompletion,
  findCompletionPrefixBeforeCursor,
  findColonPrefixBeforeCursor,
  findWordPrefixBeforeCursor,
  getEmojiSuggestions,
  isInsidePepsString,
  provideEmojiCompletionItems
} from "./emojiAutocomplete";

describe("emoji autocomplete helpers", () => {
  it("extracts a lowercase colon prefix before the cursor", () => {
    const line = "📢 :happ";
    const result = findColonPrefixBeforeCursor(line, line.length + 1);

    expect(result).toEqual({
      prefix: "happ",
      startColumn: 4,
      endColumn: 9
    });
  });

  it("extracts uppercase prefixes", () => {
    const line = "📢 :Happ";
    expect(findColonPrefixBeforeCursor(line, line.length + 1)).toEqual({
      prefix: "Happ",
      startColumn: 4,
      endColumn: 9
    });
  });

  it("extracts a plain word prefix before the cursor", () => {
    const line = "📢 length";
    expect(findWordPrefixBeforeCursor(line, line.length + 1)).toEqual({
      prefix: "length",
      startColumn: 4,
      endColumn: 10
    });
  });

  it("prefers colon prefixes when both styles are possible", () => {
    const line = "📢 :length";
    expect(findCompletionPrefixBeforeCursor(line, line.length + 1)).toEqual({
      prefix: "length",
      startColumn: 4,
      endColumn: 11
    });
  });

  it("returns at most ten happy suggestions with prefix matches", () => {
    const suggestions = getEmojiSuggestions("happ");

    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions.length).toBeLessThanOrEqual(10);
    for (const suggestion of suggestions) {
      const aliasesAndName = [suggestion.name, ...suggestion.aliases];
      const keywordMatch = suggestion.keywords.some((keyword) =>
        keyword.toLowerCase().startsWith("happ")
      );
      const nameMatch = aliasesAndName.some((item) =>
        item.toLowerCase().startsWith("happ")
      );
      expect(nameMatch || keywordMatch).toBe(true);
    }
  });

  it("prioritizes Peps syntax suggestions", () => {
    const [first] = getEmojiSuggestions("print");
    expect(first.emoji).toBe("📢");
    expect(first.name).toBe("print");
  });

  it("finds logical operator emojis by name", () => {
    expect(getEmojiSuggestions("and").some((item) => item.emoji === "🤝")).toBe(true);
    expect(getEmojiSuggestions("or").some((item) => item.emoji === "🔀")).toBe(true);
    expect(getEmojiSuggestions("not").some((item) => item.emoji === "🚫")).toBe(true);
  });

  it("finds list operator emojis by name", () => {
    expect(getEmojiSuggestions("length").some((item) => item.emoji === "📏")).toBe(true);
    expect(getEmojiSuggestions("index").some((item) => item.emoji === "🔎")).toBe(true);
    expect(getEmojiSuggestions("append").some((item) => item.emoji === "📥")).toBe(true);
  });

  it("returns Peps suggestions for an empty colon prefix", () => {
    const suggestions = getEmojiSuggestions("");
    expect(suggestions.some((item) => item.emoji === "📏")).toBe(true);
    expect(suggestions.some((item) => item.emoji === "🔎")).toBe(true);
    expect(suggestions.some((item) => item.emoji === "📥")).toBe(true);
  });

  it("replaces the entire colon token", () => {
    const line = "📢 :happy";
    const range = findColonPrefixBeforeCursor(line, line.length + 1);

    expect(range).not.toBeNull();
    expect(applyEmojiCompletion(line, range!, "😊")).toBe("📢 😊");
  });

  it("detects Peps string literals on the current line", () => {
    const line = "🐶 🟰 💬 hello :happ";
    expect(isInsidePepsString(line, line.length + 1)).toBe(true);
  });

  it("keeps text matching via filterText even when label starts with emoji", () => {
    const monaco = {
      languages: {
        CompletionItemKind: {
          Text: 1,
          Keyword: 2
        }
      },
      Range: class {
        constructor(
          public startLineNumber: number,
          public startColumn: number,
          public endLineNumber: number,
          public endColumn: number
        ) {}
      }
    } as any;

    const model = {
      getLineContent: () => ":print"
    } as any;

    const completion = provideEmojiCompletionItems(monaco, model, {
      lineNumber: 1,
      column: 7
    } as any) as { suggestions: Array<{ label: string; filterText?: string }> };

    const printSuggestion = completion.suggestions.find((item) =>
      item.label.includes("print")
    );

    expect(printSuggestion).toBeDefined();
    expect(printSuggestion?.filterText).toContain("print");
    expect(printSuggestion?.filterText).toContain(":print");
  });

  it("provides completions for plain word prefixes too", () => {
    const monaco = {
      languages: {
        CompletionItemKind: {
          Text: 1,
          Keyword: 2
        }
      },
      Range: class {
        constructor(
          public startLineNumber: number,
          public startColumn: number,
          public endLineNumber: number,
          public endColumn: number
        ) {}
      }
    } as any;

    const model = {
      getLineContent: () => "length"
    } as any;

    const completion = provideEmojiCompletionItems(monaco, model, {
      lineNumber: 1,
      column: 7
    } as any) as { suggestions: Array<{ label: string }> };

    expect(completion.suggestions.some((item) => item.label.includes("📏 length"))).toBe(true);
  });

  it("provides suggestions immediately after a colon", () => {
    const monaco = {
      languages: {
        CompletionItemKind: {
          Text: 1,
          Keyword: 2
        }
      },
      Range: class {
        constructor(
          public startLineNumber: number,
          public startColumn: number,
          public endLineNumber: number,
          public endColumn: number
        ) {}
      }
    } as any;

    const model = {
      getLineContent: () => ":"
    } as any;

    const completion = provideEmojiCompletionItems(monaco, model, {
      lineNumber: 1,
      column: 2
    } as any) as { suggestions: Array<{ label: string }> };

    expect(completion.suggestions.some((item) => item.label.includes("📏 length"))).toBe(true);
  });

  it("supports uppercase emoji search prefixes", () => {
    const suggestions = getEmojiSuggestions("Prin");
    expect(suggestions.some((item) => item.name === "print")).toBe(true);
  });

  it("matches normalized query without punctuation separators", () => {
    const suggestions = getEmojiSuggestions("raisedbackofhand");
    expect(suggestions.length).toBeGreaterThan(0);
  });

  it("expands semantic query terms like air", () => {
    const suggestions = getEmojiSuggestions("air");
    expect(suggestions.length).toBeGreaterThan(0);
  });

  it("prefers direct name matches over keyword-only matches", () => {
    const suggestions = getEmojiSuggestions("air");
    const firstNames = suggestions.slice(0, 3).map((item) => item.name);
    expect(firstNames).toContain("airplane");
  });

  it("supports small typos in query text", () => {
    const suggestions = getEmojiSuggestions("rovket");
    expect(suggestions.some((item) => item.name === "rocket")).toBe(true);
  });
});
