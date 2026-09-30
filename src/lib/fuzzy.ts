export type FuzzyMatch = {
  score: number;
};

export function fuzzyMatch(query: string, text: string): FuzzyMatch | null {
  const needle = query.trim().toLowerCase();
  if (!needle) return { score: 0 };

  const haystack = text.toLowerCase();
  let needleIndex = 0;
  let score = 0;
  let lastMatch = -1;

  for (let index = 0; index < haystack.length && needleIndex < needle.length; index += 1) {
    if (haystack[index] !== needle[needleIndex]) continue;

    score += lastMatch === index - 1 ? 3 : 1;
    if (index === 0 || /[\s\-_/.]/.test(haystack[index - 1])) score += 2;
    lastMatch = index;
    needleIndex += 1;
  }

  if (needleIndex < needle.length) return null;

  return { score: score - (haystack.length - needle.length) * 0.01 };
}
