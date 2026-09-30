export type FuzzyMatch = {
  score: number;
  positions: number[];
};

export function fuzzyMatch(query: string, text: string): FuzzyMatch | null {
  const needle = query.trim().toLowerCase();
  if (!needle) return { score: 0, positions: [] };

  const haystack = text.toLowerCase();
  const positions: number[] = [];
  let needleIndex = 0;
  let score = 0;
  let lastMatch = -1;

  for (let index = 0; index < haystack.length && needleIndex < needle.length; index += 1) {
    if (haystack[index] !== needle[needleIndex]) continue;

    score += lastMatch === index - 1 ? 3 : 1;
    if (index === 0 || /[\s\-_/.]/.test(haystack[index - 1])) score += 2;
    lastMatch = index;
    positions.push(index);
    needleIndex += 1;
  }

  if (needleIndex < needle.length) return null;

  return { score: score - (haystack.length - needle.length) * 0.01, positions };
}

export type Highlight = { text: string; match: boolean };

export function highlight(query: string, text: string): Highlight[] {
  const match = fuzzyMatch(query, text);
  if (!match || match.positions.length === 0) return [{ text, match: false }];

  const matched = new Set(match.positions);
  const segments: Highlight[] = [];
  let buffer = text[0] ?? "";
  let isMatch = matched.has(0);

  for (let index = 1; index < text.length; index += 1) {
    const nextMatch = matched.has(index);
    if (nextMatch === isMatch) {
      buffer += text[index];
    } else {
      segments.push({ text: buffer, match: isMatch });
      buffer = text[index];
      isMatch = nextMatch;
    }
  }
  if (buffer) segments.push({ text: buffer, match: isMatch });
  return segments;
}
