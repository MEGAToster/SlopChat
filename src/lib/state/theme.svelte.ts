import { loadUserThemes, setStyle as setStyleCommand, setTheme as setThemeCommand, setWindowStyle } from "../ipc";
import { setMermaidScheme } from "../markdown/mermaid";
import { applyTheme, resolveTheme } from "../theme/apply";
import { BUILTIN_THEMES } from "../theme/builtin";
import type { ThemeInfo } from "../types";
import { settings } from "./settings.svelte";

const state = $state({ user: [] as ThemeInfo[] });
let styleChain: Promise<unknown> = Promise.resolve();

export function themes(): ThemeInfo[] {
  return [...BUILTIN_THEMES, ...state.user];
}

export async function loadUser(): Promise<void> {
  state.user = await loadUserThemes();
}

export function applyThemeFor(name: string): string[] {
  const map = new Map(themes().map((entry) => [entry.name, entry.theme]));
  const resolved = resolveTheme(map, name);
  const current = settings();
  const style = current.style === "auto" ? resolved.style ?? "minimal" : current.style;
  const font = { ...resolved.font, ...(current.font ?? {}) };

  const invalid = applyTheme({ ...resolved, style, font });
  setMermaidScheme(resolved.scheme);
  // Chain so rapid previews apply in order; otherwise the last IPC to resolve
  // can win with a stale window style.
  styleChain = styleChain
    .then(() => setWindowStyle(resolved.scheme, resolved.vibrancy))
    .catch(() => {});
  return invalid;
}

export async function setTheme(name: string): Promise<void> {
  applyThemeFor(name);
  await setThemeCommand(name);
}

export async function setStyle(style: string): Promise<void> {
  await setStyleCommand(style);
}
