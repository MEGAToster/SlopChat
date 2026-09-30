import { fuzzyMatch } from "../fuzzy";
import {
  listChats,
  listModels,
  openSettingsFile,
  reloadConfig,
  searchChats,
  toggleSticky,
} from "../ipc";
import type { Chat, ProviderModel } from "../types";
import { activeChatId, pinChat } from "./chat.svelte";
import { settings } from "./settings.svelte";
import { openFromProvider, openPanel } from "./settingsPanel.svelte";
import {
  isExpanded,
  newChatWithProfile,
  openChatWithProfile,
  setMode,
  setModelOverride,
  setProfile,
} from "./ui.svelte";

export type PaletteItem =
  | { kind: "action"; id: string; label: string }
  | { kind: "chat"; id: string; label: string }
  | { kind: "model"; id: string; label: string }
  | { kind: "provider"; id: string; label: string; provider: string }
  | { kind: "profile"; id: string; label: string };

type Mode = "root" | "add";

const ACTIONS: { id: string; label: string }[] = [
  { id: "new-chat", label: "New chat" },
  { id: "toggle-expanded", label: "Toggle expanded" },
  { id: "toggle-sticky", label: "Toggle sticky" },
  { id: "pin-chat", label: "Pin / unpin chat" },
  { id: "add-models", label: "Add models…" },
  { id: "settings-panel", label: "Settings" },
  { id: "config-file", label: "Edit config file" },
  { id: "reload", label: "Reload config" },
];

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

export function items(): PaletteItem[] {
  const query = state.query.trim();
  const candidates = state.mode === "add" ? providerItems() : rootItems();

  if (!query) return candidates;

  return candidates
    .map((item) => ({ item, match: fuzzyMatch(query, item.label) }))
    .filter((entry) => entry.match !== null)
    .sort((a, b) => b.match!.score - a.match!.score)
    .map((entry) => entry.item);
}

function rootItems(): PaletteItem[] {
  const seen = new Set<string>();
  const models: PaletteItem[] = [];
  for (const model of settings().models) {
    if (seen.has(model)) continue;
    seen.add(model);
    models.push({ kind: "model", id: model, label: model });
  }
  for (const profile of settings().profiles) {
    if (seen.has(profile.model)) continue;
    seen.add(profile.model);
    models.push({ kind: "model", id: profile.model, label: profile.model });
  }

  return [
    ...ACTIONS.map((action) => ({ kind: "action" as const, ...action })),
    ...state.chats.map((chat) => ({
      kind: "chat" as const,
      id: chat.id,
      label: chat.title || "New chat",
    })),
    ...models,
    ...settings().profiles.map((profile) => ({
      kind: "profile" as const,
      id: profile.name,
      label: profile.name,
    })),
  ];
}

function providerItems(): PaletteItem[] {
  return state.providerModels
    .slice()
    .sort((a, b) => a.provider.localeCompare(b.provider) || a.id.localeCompare(b.id))
    .map((model) => ({
      kind: "provider" as const,
      id: `${model.provider}/${model.id}`,
      label: model.name || model.id,
      provider: model.provider,
    }));
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
}

export function select(index: number): void {
  state.index = index;
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
      openFromProvider(item.provider, item.id);
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
