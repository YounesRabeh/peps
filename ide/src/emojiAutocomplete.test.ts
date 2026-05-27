import { describe, expect, it } from "vitest";
import {
  applyEmojiCompletion,
  findColonPrefixBeforeCursor,
  getEmojiSuggestions,
  isInsidePepsString
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

  it("rejects uppercase prefixes", () => {
    const line = "📢 :Happ";
    expect(findColonPrefixBeforeCursor(line, line.length + 1)).toBeNull();
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
});
