import { DEFAULT_KEYBINDS } from "../keymap/defaults";
import { getConfigError, getSettings, onConfigChanged, onConfigError } from "../ipc";
import type { Settings } from "../types";

export const DEFAULT_SETTINGS: Settings = {
  keybinds: DEFAULT_KEYBINDS,
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
