import { getConfigError, getSettings, onConfigChanged, onConfigError } from "../ipc";
import type { Settings } from "../types";

export const DEFAULT_SETTINGS: Settings = {
  keybinds: {
    toggle: ["Alt+Space"],
    palette: ["Ctrl+P"],
    sidebar: ["Ctrl+H"],
    prevChat: ["Ctrl+K"],
    nextChat: ["Ctrl+J"],
    cycleProfile: ["Shift+Tab"],
    sticky: ["Ctrl+Shift+P"],
    stop: ["Ctrl+C"],
    scrollUp: ["Ctrl+U"],
    scrollDown: ["Ctrl+D"],
    webToggle: ["Ctrl+W"],
    reasoning: ["Ctrl+R"],
  },
  providers: {},
  profiles: [],
  models: [],
  startup: "last",
  resumeWithinMinutes: 10,
  theme: "default-dark",
  style: "auto",
  window: { compactWidth: 680, expandedScreenRatio: 0.7 },
};

const store = $state<{ value: Settings; error: string | null }>({
  value: DEFAULT_SETTINGS,
  error: null,
});

export function settings(): Settings {
  return store.value;
}

export function configError(): string | null {
  return store.error;
}

export function setSettings(next: Settings): void {
  store.value = next;
}

export function setConfigError(message: string | null): void {
  store.error = message;
}

export async function initSettings(): Promise<void> {
  setSettings(await getSettings());
  setConfigError(await getConfigError());
  await onConfigChanged((next) => {
    setSettings(next);
    setConfigError(null);
  });
  await onConfigError(setConfigError);
}
