import rawEmojiLib from "emojilib";
const PEPS_SUGGESTIONS = [
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
    entry("🤝", "and", ["and", "both", "all"], [], "Peps operator"),
    entry("🔀", "or", ["or", "either", "any"], [], "Peps operator"),
    entry("🚫", "not", ["not", "negate", "invert"], [], "Peps operator"),
    entry("📏", "length", ["length", "len", "size"], ["count", "list"], "Peps operator"),
    entry("🔎", "index", ["index", "get", "item"], ["lookup", "position"], "Peps operator"),
    entry("📥", "append", ["append", "push", "insert"], ["list", "end", "add"], "Peps operator"),
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
const FALLBACK_EMOJIS = [
    entry("😊", "happy", ["happy"], ["smile", "joy"], "Emoji"),
    entry("😄", "smile", ["smile"], ["happy", "joy"], "Emoji"),
    entry("😁", "grin", ["grin"], ["happy", "smile"], "Emoji")
];
function entry(emoji, name, aliases, keywords, detail) {
    return { emoji, name, aliases, keywords, detail };
}
export function findColonPrefixBeforeCursor(line, column) {
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
export function findWordPrefixBeforeCursor(line, column) {
    const beforeCursor = line.slice(0, column - 1);
    const match = /(^|[^A-Za-z0-9_])([A-Za-z][A-Za-z0-9_]*)$/.exec(beforeCursor);
    if (!match) {
        return null;
    }
    const prefix = match[2];
    return {
        prefix,
        startColumn: beforeCursor.length - prefix.length + 1,
        endColumn: column
    };
}
export function findCompletionPrefixBeforeCursor(line, column) {
    return (findColonPrefixBeforeCursor(line, column) ??
        findWordPrefixBeforeCursor(line, column));
}
export function isInsidePepsString(line, column) {
    const beforeCursor = line.slice(0, column - 1);
    const delimiterCount = [...beforeCursor.matchAll(/💬/g)].length;
    return delimiterCount % 2 === 1;
}
export function applyEmojiCompletion(line, range, emoji) {
    return (line.slice(0, range.startColumn - 1) +
        emoji +
        line.slice(range.endColumn - 1));
}
export function getEmojiSuggestions(prefix) {
    const normalizedPrefix = prefix.toLowerCase();
    if (/[^a-zA-Z0-9_]/.test(prefix)) {
        return [];
    }
    const ranked = uniqueByEmoji([...PEPS_SUGGESTIONS, ...allGeneralEmojis()])
        .map((candidate) => ({
        candidate,
        match: candidateMatch(candidate, normalizedPrefix)
    }))
        .filter((item) => item.match.score > 0)
        .sort((left, right) => {
        if (right.match.score !== left.match.score) {
            return right.match.score - left.match.score;
        }
        const leftLen = left.match.length;
        const rightLen = right.match.length;
        if (leftLen !== rightLen) {
            return leftLen - rightLen;
        }
        return left.candidate.name.localeCompare(right.candidate.name);
    })
        .slice(0, 10)
        .map((item) => item.candidate);
    return ranked;
}
export function provideEmojiCompletionItems(monaco, model, position) {
    const line = model.getLineContent(position.lineNumber);
    if (isInsidePepsString(line, position.column)) {
        return { suggestions: [], incomplete: true };
    }
    const completionPrefix = findCompletionPrefixBeforeCursor(line, position.column);
    if (!completionPrefix) {
        return { suggestions: [], incomplete: true };
    }
    const suggestions = getEmojiSuggestions(completionPrefix.prefix).map((suggestion) => {
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
            kind: suggestion.detail === "Emoji"
                ? monaco.languages.CompletionItemKind.Text
                : monaco.languages.CompletionItemKind.Keyword,
            detail: suggestion.detail,
            insertText: suggestion.emoji,
            range: new monaco.Range(position.lineNumber, completionPrefix.startColumn, position.lineNumber, completionPrefix.endColumn)
        };
    });
    return { suggestions, incomplete: true };
}
function candidateMatch(candidate, query) {
    const normalizedQuery = normalizeLookupText(query);
    if (!normalizedQuery) {
        return { score: 0, length: Number.MAX_SAFE_INTEGER };
    }
    const queryTokens = tokenizeLookupText(query);
    const fields = [
        { text: candidate.name, weight: 1.0 },
        ...candidate.aliases.map((alias) => ({ text: alias, weight: 0.9 })),
        ...candidate.keywords.map((keyword) => ({ text: keyword, weight: 0.6 }))
    ];
    let best = 0;
    let bestLength = Number.MAX_SAFE_INTEGER;
    for (const field of fields) {
        const fieldMatch = scoreField(field.text, normalizedQuery, queryTokens);
        const weightedScore = Math.round(fieldMatch.score * field.weight);
        if (weightedScore > best) {
            best = weightedScore;
            bestLength = fieldMatch.length;
        }
        else if (weightedScore === best) {
            bestLength = Math.min(bestLength, fieldMatch.length);
        }
    }
    if (candidate.detail !== "Emoji" && best > 0) {
        best += 10;
    }
    return { score: best, length: bestLength };
}
function scoreField(field, normalizedQuery, queryTokens) {
    const normalizedField = normalizeLookupText(field);
    if (!normalizedField) {
        return { score: 0, length: Number.MAX_SAFE_INTEGER };
    }
    const fieldTokens = tokenizeLookupText(field);
    const tokenStarts = fieldTokens.some((token) => token.startsWith(normalizedQuery));
    const tokenExact = fieldTokens.some((token) => token === normalizedQuery);
    const compactContains = normalizedField.includes(normalizedQuery);
    const queryLooksLikeShortToken = normalizedQuery.length <= 3;
    if (normalizedField === normalizedQuery || tokenExact) {
        return { score: 1200, length: normalizedField.length };
    }
    if (normalizedField.startsWith(normalizedQuery)) {
        return { score: 1000, length: normalizedField.length };
    }
    if (tokenStarts) {
        const matchedToken = fieldTokens.find((token) => token.startsWith(normalizedQuery));
        return { score: 900, length: matchedToken?.length ?? normalizedField.length };
    }
    // For very short queries, avoid noisy broad substring matches.
    if (!queryLooksLikeShortToken && compactContains) {
        return { score: 600, length: normalizedField.length };
    }
    // Fallback: each query token should be prefix-matched by some field token.
    if (queryTokens.length > 1 &&
        queryTokens.every((queryToken) => fieldTokens.some((fieldToken) => fieldToken.startsWith(queryToken)))) {
        return { score: 700, length: normalizedField.length };
    }
    // Typo tolerance: allow close edits for longer single-token queries.
    if (normalizedQuery.length >= 4) {
        const bestDistance = minTokenDistance(normalizedQuery, fieldTokens);
        const maxDistance = normalizedQuery.length >= 7 ? 2 : 1;
        if (bestDistance <= maxDistance) {
            return {
                score: bestDistance === 0 ? 0 : 500 - bestDistance * 80,
                length: normalizedField.length
            };
        }
    }
    return { score: 0, length: Number.MAX_SAFE_INTEGER };
}
function minTokenDistance(query, fieldTokens) {
    let best = Number.MAX_SAFE_INTEGER;
    for (const token of fieldTokens) {
        if (!token) {
            continue;
        }
        const distance = levenshteinDistance(query, token);
        if (distance < best) {
            best = distance;
        }
    }
    return best;
}
function levenshteinDistance(left, right) {
    if (left === right) {
        return 0;
    }
    if (!left.length) {
        return right.length;
    }
    if (!right.length) {
        return left.length;
    }
    const previous = Array.from({ length: right.length + 1 }, (_, idx) => idx);
    const current = new Array(right.length + 1);
    for (let i = 1; i <= left.length; i += 1) {
        current[0] = i;
        for (let j = 1; j <= right.length; j += 1) {
            const substitutionCost = left[i - 1] === right[j - 1] ? 0 : 1;
            current[j] = Math.min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + substitutionCost);
        }
        for (let j = 0; j <= right.length; j += 1) {
            previous[j] = current[j];
        }
    }
    return previous[right.length];
}
function tokenizeLookupText(text) {
    return text
        .toLowerCase()
        .split(/[^a-z0-9]+/)
        .map((token) => normalizeLookupText(token))
        .filter(Boolean);
}
function normalizeLookupText(text) {
    return text.toLowerCase().replace(/[^a-z0-9]/g, "");
}
let cachedGeneralEmojis = null;
function allGeneralEmojis() {
    if (!cachedGeneralEmojis) {
        cachedGeneralEmojis = uniqueByEmoji([
            ...normalizeEmojiLib(rawEmojiLib),
            ...FALLBACK_EMOJIS
        ]);
    }
    return cachedGeneralEmojis;
}
function normalizeEmojiLib(raw) {
    const source = unwrapEmojiLib(raw);
    if (!source || typeof source !== "object") {
        return [];
    }
    if (Array.isArray(source)) {
        return source.flatMap(normalizeArrayEntry);
    }
    return Object.entries(source).flatMap(([key, value]) => normalizeObjectEntry(key, value));
}
function unwrapEmojiLib(raw) {
    if (raw && typeof raw === "object") {
        const record = raw;
        return record.default ?? record.lib ?? raw;
    }
    return raw;
}
function normalizeArrayEntry(value) {
    if (!value || typeof value !== "object") {
        return [];
    }
    const record = value;
    const emoji = stringValue(record.emoji ?? record.char);
    if (!emoji) {
        return [];
    }
    const aliases = stringArray(record.aliases ?? record.names);
    const keywords = stringArray(record.keywords);
    const name = stringValue(record.name) ?? aliases[0] ?? keywords[0] ?? emoji;
    return [entry(emoji, name, aliases, keywords, "Emoji")];
}
function normalizeObjectEntry(key, value) {
    if (Array.isArray(value)) {
        const names = value.filter((item) => typeof item === "string");
        const name = names[0] ?? key;
        return [entry(key, name, [name], names.slice(1), "Emoji")];
    }
    if (value && typeof value === "object") {
        const record = value;
        const emoji = stringValue(record.emoji ?? record.char) ?? key;
        const aliases = stringArray(record.aliases ?? record.names);
        const keywords = stringArray(record.keywords);
        const name = stringValue(record.name) ?? aliases[0] ?? keywords[0] ?? key;
        return [entry(emoji, name, aliases, keywords, "Emoji")];
    }
    return [];
}
function stringValue(value) {
    return typeof value === "string" ? value : null;
}
function stringArray(value) {
    if (!Array.isArray(value)) {
        return [];
    }
    return value.filter((item) => typeof item === "string");
}
function uniqueByEmoji(candidates) {
    const seen = new Set();
    const unique = [];
    for (const candidate of candidates) {
        if (!seen.has(candidate.emoji)) {
            seen.add(candidate.emoji);
            unique.push(candidate);
        }
    }
    return unique;
}
