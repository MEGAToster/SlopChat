import type { Theme, ThemeStyles } from "../types";

export type ResolvedTheme = {
  name: string;
  scheme: "dark" | "light";
  style?: ThemeStyles;
  vibrancy: string;
  opacity: number;
  colors: Record<string, string>;
  syntax: Record<string, string>;
  palette: Record<string, string>;
  font: { ui?: string; mono?: string; size?: number };
};

export function resolveTheme(themes: Map<string, Theme>, name: string): ResolvedTheme {
  const chain: Theme[] = [];
  const seen = new Set<string>();
  let current: string | undefined = name;

  while (current && !seen.has(current)) {
    seen.add(current);
    const theme = themes.get(current);
    if (!theme) break;
    chain.unshift(theme);
    current = theme.extends ?? `default-${theme.scheme ?? "dark"}`;
  }

  const merged: Theme = { colors: {}, syntax: {}, palette: {}, font: {} };
  for (const theme of chain) mergeTheme(merged, theme);

  return {
    name,
    scheme: merged.scheme ?? "dark",
    style: merged.style,
    vibrancy: merged.vibrancy ?? "none",
    opacity: merged.opacity ?? 1,
    colors: merged.colors ?? {},
    syntax: merged.syntax ?? {},
    palette: merged.palette ?? {},
    font: merged.font ?? {},
  };
}

function mergeTheme(target: Theme, source: Theme): void {
  if (source.scheme) target.scheme = source.scheme;
  if (source.style) target.style = source.style;
  if (source.extends) target.extends = source.extends;
  if (source.vibrancy) target.vibrancy = source.vibrancy;
  if (source.opacity !== undefined) target.opacity = source.opacity;
  target.colors = { ...target.colors, ...source.colors };
  target.syntax = { ...target.syntax, ...source.syntax };
  target.palette = { ...target.palette, ...source.palette };
  target.font = { ...target.font, ...source.font };
}

const applied = new Set<string>();

export function applyTheme(resolved: ResolvedTheme): string[] {
  const root = document.documentElement;
  const invalid: string[] = [];
  const next: [string, string][] = [];

  for (const [key, value] of Object.entries(resolved.colors)) {
    if (isValue(value)) next.push([key, value]);
    else invalid.push(`colors.${key}`);
  }
  for (const [key, value] of Object.entries(resolved.syntax)) {
    if (isValue(value)) next.push([`syn-${key}`, value]);
    else invalid.push(`syntax.${key}`);
  }
  for (const [key, value] of Object.entries(resolved.palette)) {
    if (isValue(value)) next.push([`c-${key}`, value]);
    else invalid.push(`palette.${key}`);
  }

  const rgb = resolved.colors.bg ? hexToRgb(resolved.colors.bg) : null;
  if (rgb) next.push(["bg-rgb", rgb.join(" ")]);

  if (resolved.font.ui) next.push(["font-ui", resolved.font.ui]);
  if (resolved.font.mono) next.push(["font-mono", resolved.font.mono]);
  if (resolved.font.size) next.push(["fs-md", `${resolved.font.size}px`]);
  next.push(["bg-opacity", String(resolved.opacity)]);

  for (const key of applied) root.style.removeProperty(`--${key}`);
  applied.clear();

  for (const [key, value] of next) {
    root.style.setProperty(`--${key}`, value);
    applied.add(key);
  }

  root.dataset.style = resolved.style ?? "minimal";
  root.style.colorScheme = resolved.scheme;

  return invalid;
}

function isValue(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function hexToRgb(hex: string): [number, number, number] | null {
  const match = hex.trim().match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
  if (!match) return null;
  let value = match[1];
  if (value.length === 3) value = value[0] + value[0] + value[1] + value[1] + value[2] + value[2];
  const int = Number.parseInt(value, 16);
  return [(int >> 16) & 255, (int >> 8) & 255, int & 255];
}
