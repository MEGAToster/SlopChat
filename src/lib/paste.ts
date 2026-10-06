// A pasted block at or above the limit becomes a text attachment. 0 disables it.
export function shouldPasteAsFile(text: string, limit: number): boolean {
  return limit > 0 && text.length >= limit;
}
