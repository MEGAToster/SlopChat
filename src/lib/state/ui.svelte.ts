import { setChatProfile, setPanelExpanded } from "../ipc";
import type { Context } from "../keymap/dispatch";
import type { Chat, Profile } from "../types";
import {
  activeChatId,
  activeChatIsEmpty,
  chats,
  newChat,
  openChat,
  setCurrentProfile,
} from "./chat.svelte";
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
  if (name) setCurrentProfile(name);
  const index = settings().profiles.findIndex((profile) => profile.name === name);
  if (index !== -1) store.profileIndex = index;
}

export function cycleProfile(): void {
  const profiles = settings().profiles;
  if (profiles.length === 0) return;
  store.profileIndex = (store.profileIndex + 1) % profiles.length;

  const name = profiles[store.profileIndex].name;
  setCurrentProfile(name);

  const chatId = activeChatId();
  if (chatId) void setChatProfile(chatId, name);
}

export async function openChatWithProfile(chatId: string): Promise<void> {
  await openChat(chatId);
  syncProfileFromActive();
  store.context = "composer";
}

export async function newChatWithProfile(): Promise<void> {
  if (activeChatId() !== null && activeChatIsEmpty()) return;

  const profile = activeProfile();
  if (profile) setCurrentProfile(profile.name);
  await newChat();
  syncProfileFromActive();
  store.context = "composer";
}

export function syncProfileFromActive(): void {
  const chat = chats().find((candidate) => candidate.id === activeChatId());
  if (chat) setProfile(chat.profile);
}
