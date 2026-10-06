// atob yields one char per byte; decode those bytes as UTF-8 so non-ASCII
// content (umlauts, emoji) survives instead of turning into mojibake.
export function decodeBase64(data: string): string {
  try {
    const bytes = Uint8Array.from(atob(data), (char) => char.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  } catch {
    return "";
  }
}
