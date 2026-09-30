import { resizePanel, setChatProfile, setPanelExpanded } from "../ipc";
import type { Context } from "../keymap/dispatch";
import type { Chat, Profile } from "../types";
import { activeChatId, chats, isBlank, newChat, openChat } from "./chat.svelte";
import { settings } from "./settings.svelte";

type PanelMode = "compact" | "expanded";

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
});

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
  if (!value) store.filterQuery = "";
  store.selectedIndex = 0;
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

export function measurePanel(): void {
  if (store.mode === "expanded") return;
  const content = document.querySelector<HTMLElement>(".content");
  if (content) void resizePanel(Math.ceil(content.getBoundingClientRect().height));
}

export function visibleChats(): Chat[] {
  const query = store.filterQuery.trim().toLowerCase();
  const all = chats();
  if (!query) return all;
  return all.filter((chat) => chat.title.toLowerCase().includes(query));
}

export function moveSelection(delta: number): void {
  const count = visibleChats().length;
  if (count === 0) return;
  store.selectedIndex = Math.min(Math.max(store.selectedIndex + delta, 0), count - 1);
}

export function selectedChatId(): string | null {
  return visibleChats()[store.selectedIndex]?.id ?? null;
}

export function selectActive(): void {
  const index = visibleChats().findIndex((chat) => chat.id === activeChatId());
  store.selectedIndex = index === -1 ? 0 : index;
}

export function clampSelection(): void {
  const count = visibleChats().length;
  store.selectedIndex = count === 0 ? 0 : Math.min(store.selectedIndex, count - 1);
}

export function activeProfile(): Profile | undefined {
  const profiles = settings().profiles;
  if (profiles.length === 0) return undefined;
  return profiles[store.profileIndex % profiles.length];
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

export async function previewChatWithProfile(chatId: string): Promise<void> {
  if (activeChatId() === chatId) return;
  await openChat(chatId);
  syncProfileFromActive();
}

export async function newChatWithProfile(): Promise<void> {
  if (isBlank()) return;
  await newChat();
  store.modelOverride = null;
  store.context = "composer";
}

export function syncProfileFromActive(): void {
  const chat = chats().find((candidate) => candidate.id === activeChatId());
  if (chat) setProfile(chat.profile);
}
