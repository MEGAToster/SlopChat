// Some models emit their tool-call protocol (e.g. DeepSeek's `<|DSML| …>`) as
// plain text when the provider doesn't parse it. Drop it so it never renders.
export function stripControlTokens(text: string): string {
  return text
    .replace(/<\|\s*DSML\s*\|\s*calls\s*>[\s\S]*?<\/\|\s*DSML\s*\|\s*calls\s*>/gi, "")
    .replace(/<\/?\|\s*DSML\s*\|[^>]*>/gi, "")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}
