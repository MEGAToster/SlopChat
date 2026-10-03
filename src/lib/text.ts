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
