import { loadUserThemes, setTheme as setThemeCommand, setWindowStyle } from "../ipc";
import { applyTheme, resolveTheme } from "../theme/apply";
import { BUILTIN_THEMES } from "../theme/builtin";
import type { ThemeInfo } from "../types";

const state = $state({ user: [] as ThemeInfo[] });

export function themes(): ThemeInfo[] {
  return [...BUILTIN_THEMES, ...state.user];
}

export async function loadUser(): Promise<void> {
  state.user = await loadUserThemes();
}

export function applyThemeFor(name: string): string[] {
  const map = new Map(themes().map((entry) => [entry.name, entry.theme]));
  const resolved = resolveTheme(map, name);
  const invalid = applyTheme(resolved);
  void setWindowStyle(resolved.scheme, resolved.vibrancy, resolved.opacity);
  return invalid;
}

export async function setTheme(name: string): Promise<void> {
  applyThemeFor(name);
  await setThemeCommand(name);
}
