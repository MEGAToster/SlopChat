import { DEFAULT_KEYBINDS } from "../keymap/defaults";
import { getConfigError, getSettings, onConfigChanged, onConfigError } from "../ipc";
import type { Settings } from "../types";

const DEFAULT_SETTINGS: Settings = {
  keybinds: DEFAULT_KEYBINDS,
  providers: {},
  profiles: [],
  models: [],
  startup: "latest",
  resumeWithinMinutes: 10,
  pasteAsFileChars: 10_000,
  theme: "default-dark",
  style: "auto",
  search: {
    providers: ["searxng", "exa", "tavily", "brave", "firecrawl", "serper"],
    readerFallback: true,
    searxng: { baseUrl: "" },
  },
  advanced: {
    maxSearches: 5,
    maxFetches: 50,
    maxRounds: 5,
    responseTimeoutSecs: 60,
    maxTurnSecs: 300,
    maxFetchedChars: 60000,
    reasoningMaxTokens: 0,
    frequencyPenalty: 0.2,
    presencePenalty: 0.2,
    deep: {
      maxRounds: 15,
      maxSearches: 30,
      maxFetches: 40,
      maxFetchedChars: 400000,
      maxTurnSecs: 900,
      subagentModel: "",
    },
  },
  debug: { agentLog: false },
  window: { compactWidth: 680, compactHeight: 480, expandedScreenRatio: 0.7 },
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

function setSettings(next: Settings): void {
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
