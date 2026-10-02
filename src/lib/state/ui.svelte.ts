import { groupChats, type ChatGroup } from "../chatGroups";
import { setChatProfile, setPanelExpanded } from "../ipc";
import type { Context } from "../keymap/dispatch";
import type { Chat, ModelConfig, Profile } from "../types";
import { effectiveWebMode } from "../webSearch";
import { newChat, openChat } from "./chatHistory.svelte";
import { activeChatId, chats, isBlank, webOverride, type SendOptions } from "./chatStore.svelte";
import { reasoningOverride } from "./reasoning.svelte";
import { settings } from "./settings.svelte";

export type { ChatGroup };

type PanelMode = "compact" | "expanded";

export const OVERLAY_HEIGHT = 480;

const store = $state({
  mode: "compact" as PanelMode,
  context: "composer" as Context,
  profileIndex: 0,
  selectedIndex: 0,
  filtering: false,
  filterQuery: "",
  pendingDelete: null as string | null,
  editingId: null as string | null,
  modelOverride: null as string | null,
  noticeMessage: null as string | null,
  noticeKind: "info" as "danger" | "warning" | "success" | "info",
});

export function notify(
  message: string,
  kind: "danger" | "warning" | "success" | "info" = "info",
): void {
  store.noticeKind = kind;
  store.noticeMessage = message;
}

export function uiNotice(): string | null {
  return store.noticeMessage;
}

export function uiNoticeKind(): "danger" | "warning" | "success" | "info" {
  return store.noticeKind;
}

export function clearNotice(): void {
  store.noticeMessage = null;
}

export function mode(): PanelMode {
  return store.mode;
}

export function isExpanded(): boolean {
  return store.mode === "expanded";
}

export function keyContext(): Context {
  return store.context;
}

export function selectedIndex(): number {
  return store.selectedIndex;
}

export function filtering(): boolean {
  return store.filtering;
}

export function filterQuery(): string {
  return store.filterQuery;
}

export function pendingDelete(): string | null {
  return store.pendingDelete;
}

export function editingId(): string | null {
  return store.editingId;
}

export function modelOverride(): string | null {
  return store.modelOverride;
}

export function setModelOverride(modelId: string | null): void {
  store.modelOverride = modelId;
}

export function setContext(context: Context): void {
  store.context = context;
}

export function setFiltering(value: boolean): void {
  store.filtering = value;
  if (value) {
    store.selectedIndex = 0;
  } else {
    store.filterQuery = "";
  }
}

export function setFilterQuery(value: string): void {
  store.filterQuery = value;
  store.selectedIndex = 0;
}

export function setPendingDelete(id: string | null): void {
  store.pendingDelete = id;
}

export function setSelectedIndex(index: number): void {
  store.selectedIndex = index;
}

export function beginEdit(messageId: string): void {
  store.editingId = messageId;
}

export function endEdit(): void {
  store.editingId = null;
}

export async function setMode(mode: PanelMode): Promise<void> {
  store.mode = mode;
  await setPanelExpanded(mode === "expanded");
}

function visibleChats(): Chat[] {
  const query = store.filterQuery.trim().toLowerCase();
  const all = chats();
  if (!query) return all;
  return all.filter((chat) => chat.title.toLowerCase().includes(query));
}

export function chatGroups(): ChatGroup[] {
  return groupChats(visibleChats());
}

function orderedChats(): Chat[] {
  return chatGroups().flatMap((group) => group.chats);
}

// The last selectable row is the "New chat" entry (index === count).
export function moveSelection(delta: number): void {
  const count = orderedChats().length;
  store.selectedIndex = Math.min(Math.max(store.selectedIndex + delta, 0), count);
}

export function selectedChatId(): string | null {
  return orderedChats()[store.selectedIndex]?.id ?? null;
}

export function selectActive(): void {
  const count = orderedChats().length;
  const index = orderedChats().findIndex((chat) => chat.id === activeChatId());
  store.selectedIndex = index === -1 ? count : index;
}

export function activeProfile(): Profile | undefined {
  const profiles = settings().profiles;
  if (profiles.length === 0) return undefined;
  return profiles[store.profileIndex % profiles.length];
}

export function activeProfileIndex(): number {
  return store.profileIndex;
}

// The settings for the model in play: the override's entry if one is selected,
// otherwise the active profile.
export function activeConfig(): ModelConfig | null {
  const profile = activeProfile();
  if (!profile) return null;

  const ref = modelOverride() ?? profile.model;
  const model =
    settings().models.find((entry) => entry.model === ref) ?? { model: ref, useContext: true };

  // Per-profile overrides only apply to that profile's own model; a model
  // picked from the palette uses its registered settings as-is.
  if (ref !== profile.model) return model;

  return {
    ...model,
    system: profile.system ?? model.system,
    temperature: profile.temperature ?? model.temperature,
    useContext: profile.useContext ?? model.useContext,
    tools: profile.tools ?? model.tools,
    toolCalling: profile.toolCalling ?? model.toolCalling,
    webSearch: profile.webSearch ?? model.webSearch,
    reasoning: profile.reasoning ?? model.reasoning,
  };
}

// The request options for the active profile/model, shared by send and retry.
export function currentSendOptions(): SendOptions | null {
  const profile = activeProfile();
  const config = activeConfig();
  if (!profile || !config) return null;

  const toolCalling = config.toolCalling !== false;
  return {
    profileName: profile.name,
    model: config.model,
    system: config.system,
    temperature: config.temperature,
    useContext: config.useContext,
    tools: toolCalling ? config.tools : [],
    toolCalling,
    webSearch: effectiveWebMode(toolCalling, webOverride(), config.webSearch),
    reasoning: reasoningOverride() ?? config.reasoning,
    fallback: modelOverride() ? undefined : profile.fallback,
    maxCompletionTokens: config.maxCompletionTokens,
  };
}

export function setProfile(name: string): void {
  const index = settings().profiles.findIndex((profile) => profile.name === name);
  if (index !== -1) store.profileIndex = index;
  store.modelOverride = null;
}

export function cycleProfile(): void {
  const profiles = settings().profiles;
  if (profiles.length === 0) return;
  store.profileIndex = (store.profileIndex + 1) % profiles.length;
  store.modelOverride = null;

  const name = profiles[store.profileIndex].name;

  const chatId = activeChatId();
  if (chatId) void setChatProfile(chatId, name);
}

export async function openChatWithProfile(chatId: string): Promise<void> {
  await openChat(chatId);
  syncProfileFromActive();
  store.modelOverride = null;
  store.editingId = null;
  store.context = "composer";
}

let previewTimer: ReturnType<typeof setTimeout> | undefined;

export function schedulePreview(): void {
  clearTimeout(previewTimer);
  previewTimer = setTimeout(() => {
    const id = selectedChatId();
    if (id) void previewChatWithProfile(id);
  }, 60);
}

async function previewChatWithProfile(chatId: string): Promise<void> {
  if (activeChatId() === chatId) return;
  await openChat(chatId);
  syncProfileFromActive();
  store.editingId = null;
}

export async function newChatWithProfile(): Promise<void> {
  if (!isBlank()) {
    await newChat();
    store.modelOverride = null;
    store.editingId = null;
    store.context = "composer";
  }
  selectActive();
}

export function syncProfileFromActive(): void {
  const chat = chats().find((candidate) => candidate.id === activeChatId());
  if (chat) setProfile(chat.profile);
}
