const applied = new Set<string>();

export function applyTheme(tokens: Record<string, string>): void {
  const root = document.documentElement;

  for (const key of applied) root.style.removeProperty(`--${key}`);
  applied.clear();

  for (const [key, value] of Object.entries(tokens)) {
    root.style.setProperty(`--${key}`, value);
    applied.add(key);
  }
}
