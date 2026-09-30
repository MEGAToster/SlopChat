import {
  listChats,
  listModels,
  openSettingsFile,
  reloadConfig,
  searchChats,
  searchUsage,
  toggleSticky,
} from "../ipc";
import type { Chat, ProviderModel } from "../types";
import { activeChatId, pinChat } from "./chat.svelte";
import { addGroups, rootGroups } from "./paletteItems";
import type { PaletteGroup, PaletteItem } from "./paletteTypes";
import { settings } from "./settings.svelte";
import { openFromProvider, openPanel } from "./settingsPanel.svelte";
import { applyThemeFor, setStyle, setTheme, themes } from "./theme.svelte";
import {
  isExpanded,
  newChatWithProfile,
  notify,
  openChatWithProfile,
  setMode,
  setModelOverride,
  setProfile,
} from "./ui.svelte";

export type { PaletteGroup, PaletteItem } from "./paletteTypes";

type Mode = "root" | "add";

const state = $state({
  open: false,
  mode: "root" as Mode,
  query: "",
  index: 0,
  chats: [] as Chat[],
  providerModels: [] as ProviderModel[],
  loading: false,
});

let searchTimer: ReturnType<typeof setTimeout> | undefined;

export function isOpen(): boolean {
  return state.open;
}

export function mode(): Mode {
  return state.mode;
}

export function query(): string {
  return state.query;
}

export function index(): number {
  return state.index;
}

export function loading(): boolean {
  return state.loading;
}

export function breadcrumb(): string {
  return state.mode === "add" ? "Add model" : "Commands";
}

export function groups(): PaletteGroup[] {
  if (state.mode === "add") return addGroups(state.providerModels, state.query);
  return rootGroups({
    settings: settings(),
    chats: state.chats,
    themes: themes(),
    query: state.query,
  });
}

export function items(): PaletteItem[] {
  return groups().flatMap((group) => group.items);
}

export async function open(): Promise<void> {
  state.open = true;
  state.mode = "root";
  state.query = "";
  state.index = 0;
  await refreshChats("");
}

export function close(): void {
  state.open = false;
  state.mode = "root";
  clearTimeout(searchTimer);
  applyThemeFor(settings().theme);
}

export function setQuery(value: string): void {
  state.query = value;
  state.index = 0;

  if (state.mode === "add") return;

  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => void refreshChats(value), 120);
}

export function move(delta: number): void {
  const count = items().length;
  if (count === 0) return;
  state.index = Math.min(Math.max(state.index + delta, 0), count - 1);
  previewSelected();
}

export function select(index: number): void {
  state.index = index;
  previewSelected();
}

function previewSelected(): void {
  const item = items()[state.index];
  if (item?.kind === "theme") applyThemeFor(item.id);
}

export function back(): void {
  state.mode = "root";
  state.query = "";
  state.index = 0;
}

export async function activate(selected?: number): Promise<boolean> {
  if (selected !== undefined) state.index = selected;
  const item = items()[state.index];
  if (!item) return true;

  if (item.kind === "action" && item.id === "add-models") {
    await startAddModels();
    return false;
  }

  await run(item);
  return true;
}

async function startAddModels(): Promise<void> {
  state.mode = "add";
  state.query = "";
  state.index = 0;
  if (state.providerModels.length > 0) return;

  state.loading = true;
  try {
    state.providerModels = await listModels();
  } catch {
    state.providerModels = [];
  }
  state.loading = false;
}

async function refreshChats(query: string): Promise<void> {
  try {
    state.chats = query.trim() ? await searchChats(query) : await listChats();
  } catch {
    state.chats = [];
  }
}

async function run(item: PaletteItem): Promise<void> {
  switch (item.kind) {
    case "action":
      await runAction(item.id);
      break;
    case "chat":
      await openChatWithProfile(item.id);
      break;
    case "profile":
      setProfile(item.id);
      break;
    case "model":
      setModelOverride(item.id);
      break;
    case "provider":
      openFromProvider(item.provider ?? "", item.id);
      break;
    case "theme":
      await setTheme(item.id);
      break;
  }
}

async function runAction(id: string): Promise<void> {
  switch (id) {
    case "new-chat":
      await newChatWithProfile();
      break;
    case "toggle-expanded":
      await setMode(isExpanded() ? "compact" : "expanded");
      break;
    case "toggle-sticky":
      await toggleSticky();
      break;
    case "pin-chat": {
      const chatId = activeChatId();
      if (chatId) await pinChat(chatId);
      break;
    }
    case "switch-style":
      await setStyle(settings().style === "tui" ? "minimal" : "tui");
      break;
    case "search-usage": {
      const usage = await searchUsage();
      notify(
        `Exa: ${usage.requests} searches · $${usage.cost.toFixed(2)} / $${usage.budget.toFixed(2)} this month`,
      );
      break;
    }
    case "settings-panel":
      openPanel();
      break;
    case "config-file":
      await openSettingsFile();
      break;
    case "reload":
      await reloadConfig();
      break;
  }
}
