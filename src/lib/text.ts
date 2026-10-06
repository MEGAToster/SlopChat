export function basename(path: string): string {
  return path.split("/").pop() ?? path;
}

export function domain(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export function formatBytes(bytes: number): string {
  if (bytes < 1000) return `${bytes} B`;
  if (bytes < 1_000_000) return `${Math.round(bytes / 1000)}k`;
  return `${(bytes / 1_000_000).toFixed(1)}M`;
}
