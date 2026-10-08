// A pasted block at or above the limit becomes a text attachment. 0 disables it.
// The hard cap applies even when the limit is 0 so a huge paste never lands in
// the composer value, where it would stall layout and keydown handling.
export const PASTE_AS_FILE_HARD_CAP = 1_000_000;

export function shouldPasteAsFile(text: string, limit: number): boolean {
  if (text.length >= PASTE_AS_FILE_HARD_CAP) return true;
  return limit > 0 && text.length >= limit;
}
