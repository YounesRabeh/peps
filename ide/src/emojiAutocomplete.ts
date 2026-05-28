import rawEmojiLib from "emojilib";
import type * as Monaco from "monaco-editor";

export type EmojiSuggestion = {
  emoji: string;
  name: string;
  aliases: string[];
  keywords: string[];
  detail: "Peps keyword" | "Peps operator" | "Peps syntax" | "Emoji";
};

export type ColonPrefix = {
  prefix: string;
  startColumn: number;
  endColumn: number;
};

const PEPS_SUGGESTIONS: EmojiSuggestion[] = [
  entry("📢", "print", ["print", "say", "output"], [], "Peps keyword"),
  entry("🤔", "if", ["if", "condition"], [], "Peps keyword"),
  entry("😐", "else", ["else"], [], "Peps keyword"),
  entry("🔁", "while", ["while", "loop", "for"], [], "Peps keyword"),
  entry("🛑", "break", ["break", "stop", "exit"], [], "Peps keyword"),
  entry("⏭️", "continue", ["continue", "next", "skip"], [], "Peps keyword"),
  entry("🧭", "in", ["in"], [], "Peps keyword"),
  entry("🔢", "range", ["range", "number"], [], "Peps keyword"),
  entry("✅", "true", ["true", "yes"], [], "Peps keyword"),
  entry("❌", "false", ["false", "no", "not"], [], "Peps keyword"),
  entry("🟰", "assign", ["assign", "equals"], [], "Peps operator"),
  entry("➕", "plus", ["plus", "add"], [], "Peps operator"),
  entry("➖", "minus", ["minus", "subtract", "negative"], [], "Peps operator"),
  entry("✖️", "multiply", ["multiply", "times"], [], "Peps operator"),
  entry("➗", "divide", ["divide"], [], "Peps operator"),
  entry("➡️", "range end", ["to", "arrow"], [], "Peps operator"),
  entry("🔓", "block start", ["block", "open", "start"], [], "Peps syntax"),
  entry("🔒", "block end", ["end", "close"], [], "Peps syntax"),
  entry("🔚", "statement end", ["end", "line", "statement"], [], "Peps syntax"),
  entry("💬", "string", ["string", "text", "quote"], [], "Peps syntax"),
  entry("📚", "list", ["list", "array"], [], "Peps syntax")
];

const FALLBACK_EMOJIS: EmojiSuggestion[] = [
  entry("😊", "happy", ["happy"], ["smile", "joy"], "Emoji"),
  entry("😄", "smile", ["smile"], ["happy", "joy"], "Emoji"),
  entry("😁", "grin", ["grin"], ["happy", "smile"], "Emoji")
];

function entry(
  emoji: string,
  name: string,
  aliases: string[],
  keywords: string[],
  detail: EmojiSuggestion["detail"]
): EmojiSuggestion {
  return { emoji, name, aliases, keywords, detail };
}

export function findColonPrefixBeforeCursor(
  line: string,
  column: number
): ColonPrefix | null {
  const beforeCursor = line.slice(0, column - 1);
  const match = /:([A-Za-z0-9_]*)$/.exec(beforeCursor);
  if (!match || match.index < 0) {
    return null;
  }

  return {
    prefix: match[1],
    startColumn: match.index + 1,
    endColumn: column
  };
}

export function isInsidePepsString(line: string, column: number): boolean {
  const beforeCursor = line.slice(0, column - 1);
  const delimiterCount = [...beforeCursor.matchAll(/💬/g)].length;
  return delimiterCount % 2 === 1;
}

export function applyEmojiCompletion(
  line: string,
  range: ColonPrefix,
  emoji: string
): string {
  return (
    line.slice(0, range.startColumn - 1) +
    emoji +
    line.slice(range.endColumn - 1)
  );
}

export function getEmojiSuggestions(prefix: string): EmojiSuggestion[] {
  const normalizedPrefix = prefix.toLowerCase();
  if (/[^a-zA-Z0-9_]/.test(prefix)) {
    return [];
  }

  const ranked = uniqueByEmoji([...PEPS_SUGGESTIONS, ...allGeneralEmojis()])
    .map((candidate) => ({
      candidate,
      score: suggestionScore(candidate, normalizedPrefix)
    }))
    .filter((item) => item.score > 0)
    .sort((left, right) => right.score - left.score)
    .slice(0, 10)
    .map((item) => item.candidate);

  return ranked;
}

export function provideEmojiCompletionItems(
  monaco: typeof Monaco,
  model: Monaco.editor.ITextModel,
  position: Monaco.Position
): Monaco.languages.ProviderResult<Monaco.languages.CompletionList> {
  const line = model.getLineContent(position.lineNumber);
  if (isInsidePepsString(line, position.column)) {
    return { suggestions: [], incomplete: true };
  }

  const colonPrefix = findColonPrefixBeforeCursor(line, position.column);
  if (!colonPrefix) {
    return { suggestions: [], incomplete: true };
  }

  const suggestions = getEmojiSuggestions(colonPrefix.prefix).map((suggestion) => {
    const filterTerms = [
      suggestion.name,
      ...suggestion.aliases,
      ...suggestion.keywords
    ];

    return {
      label: `${suggestion.emoji} ${suggestion.name}`,
      filterText: [
        ...filterTerms,
        ...filterTerms.map((term) => `:${term}`)
      ].join(" "),
      kind:
        suggestion.detail === "Emoji"
          ? monaco.languages.CompletionItemKind.Text
          : monaco.languages.CompletionItemKind.Keyword,
      detail: suggestion.detail,
      insertText: suggestion.emoji,
      range: new monaco.Range(
        position.lineNumber,
        colonPrefix.startColumn,
        position.lineNumber,
        colonPrefix.endColumn
      )
    };
  });

  return { suggestions, incomplete: true };
}

function matchesAliasOrName(candidate: EmojiSuggestion, prefix: string): boolean {
  return suggestionScore(candidate, prefix) > 0;
}

function suggestionScore(candidate: EmojiSuggestion, query: string): number {
  const normalizedQuery = normalizeLookupText(query);
  if (!normalizedQuery) {
    return 0;
  }

  const fields = [candidate.name, ...candidate.aliases, ...candidate.keywords];
  let best = 0;

  for (const field of fields) {
    const normalizedField = normalizeLookupText(field);
    if (!normalizedField) {
      continue;
    }

    if (normalizedField === normalizedQuery) {
      best = Math.max(best, 100);
    } else if (normalizedField.startsWith(normalizedQuery)) {
      best = Math.max(best, 80);
    } else if (normalizedField.includes(normalizedQuery)) {
      best = Math.max(best, 50);
    }
  }

  if (candidate.detail !== "Emoji" && best > 0) {
    best += 10;
  }

  return best;
}

function normalizeLookupText(text: string): string {
  return text.toLowerCase().replace(/[^a-z0-9]/g, "");
}

let cachedGeneralEmojis: EmojiSuggestion[] | null = null;

function allGeneralEmojis(): EmojiSuggestion[] {
  if (!cachedGeneralEmojis) {
    cachedGeneralEmojis = uniqueByEmoji([
      ...normalizeEmojiLib(rawEmojiLib),
      ...FALLBACK_EMOJIS
    ]);
  }
  return cachedGeneralEmojis;
}

function normalizeEmojiLib(raw: unknown): EmojiSuggestion[] {
  const source = unwrapEmojiLib(raw);
  if (!source || typeof source !== "object") {
    return [];
  }

  if (Array.isArray(source)) {
    return source.flatMap(normalizeArrayEntry);
  }

  return Object.entries(source).flatMap(([key, value]) =>
    normalizeObjectEntry(key, value)
  );
}

function unwrapEmojiLib(raw: unknown): unknown {
  if (raw && typeof raw === "object") {
    const record = raw as Record<string, unknown>;
    return record.default ?? record.lib ?? raw;
  }
  return raw;
}

function normalizeArrayEntry(value: unknown): EmojiSuggestion[] {
  if (!value || typeof value !== "object") {
    return [];
  }
  const record = value as Record<string, unknown>;
  const emoji = stringValue(record.emoji ?? record.char);
  if (!emoji) {
    return [];
  }
  const aliases = stringArray(record.aliases ?? record.names);
  const keywords = stringArray(record.keywords);
  const name = stringValue(record.name) ?? aliases[0] ?? keywords[0] ?? emoji;
  return [entry(emoji, name, aliases, keywords, "Emoji")];
}

function normalizeObjectEntry(key: string, value: unknown): EmojiSuggestion[] {
  if (Array.isArray(value)) {
    const names = value.filter((item): item is string => typeof item === "string");
    return [entry(key, names[0] ?? key, names, names.slice(1), "Emoji")];
  }

  if (value && typeof value === "object") {
    const record = value as Record<string, unknown>;
    const emoji = stringValue(record.emoji ?? record.char) ?? key;
    const aliases = stringArray(record.aliases ?? record.names);
    const keywords = stringArray(record.keywords);
    const name = stringValue(record.name) ?? aliases[0] ?? keywords[0] ?? key;
    return [entry(emoji, name, aliases, keywords, "Emoji")];
  }

  return [];
}

function stringValue(value: unknown): string | null {
  return typeof value === "string" ? value : null;
}

function stringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string");
}

function uniqueByEmoji(candidates: EmojiSuggestion[]): EmojiSuggestion[] {
  const seen = new Set<string>();
  const unique: EmojiSuggestion[] = [];
  for (const candidate of candidates) {
    if (!seen.has(candidate.emoji)) {
      seen.add(candidate.emoji);
      unique.push(candidate);
    }
  }
  return unique;
}
