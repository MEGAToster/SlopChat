import { loadUserThemes, setTheme as setThemeCommand } from "../ipc";
import { applyTheme } from "../theme/apply";
import { BUILTIN_THEMES } from "../theme/builtin";
import type { ThemeInfo } from "../types";

const state = $state({ user: [] as ThemeInfo[] });

export function themes(): ThemeInfo[] {
  return [...BUILTIN_THEMES, ...state.user];
}

export async function loadUser(): Promise<void> {
  state.user = await loadUserThemes();
}

export function applyThemeFor(name: string): void {
  const theme = themes().find((candidate) => candidate.name === name) ?? BUILTIN_THEMES[0];
  applyTheme(theme.tokens);
}

export async function setTheme(name: string): Promise<void> {
  applyThemeFor(name);
  await setThemeCommand(name);
}
