import {
  createChat,
  deleteChat as deleteChatCommand,
  getChat,
  initialChat,
  listChats,
  toggleChatPin,
} from "../ipc";
import { chatState } from "./chatStore.svelte";
import { lastUsageOf, toChatMessage } from "./messages";

export async function init(): Promise<void> {
  const view = await initialChat();
  if (view) {
    chatState.activeChatId = view.chat.id;
    chatState.messages = view.messages.map(toChatMessage);
    chatState.usage = lastUsageOf(view.messages);
  }
  await refreshChats();
}

export async function refreshChats(): Promise<void> {
  chatState.chats = await listChats();
}

export async function openChat(chatId: string): Promise<void> {
  const view = await getChat(chatId);
  if (!view) return;
  chatState.activeChatId = view.chat.id;
  chatState.messages = view.messages.map(toChatMessage);
  chatState.usage = lastUsageOf(view.messages);
  chatState.error = null;
  await refreshChats();
}

export async function newChat(): Promise<void> {
  if (chatState.activeChatId === null && chatState.messages.length === 0) return;
  chatState.activeChatId = null;
  chatState.messages = [];
  chatState.usage = null;
  chatState.error = null;
}

export async function prevChat(): Promise<void> {
  await step(-1);
}

export async function nextChat(): Promise<void> {
  await step(1);
}

export async function pinChat(chatId: string): Promise<void> {
  await toggleChatPin(chatId);
  await refreshChats();
}

export async function removeChat(chatId: string): Promise<void> {
  const wasActive = chatState.activeChatId === chatId;
  await deleteChatCommand(chatId);
  await refreshChats();

  if (!wasActive) return;

  if (chatState.chats.length === 0) {
    await newChat();
    return;
  }

  await openChat(chatState.chats[chatState.chats.length - 1].id);
}

export async function ensureChat(profileName: string): Promise<string> {
  if (chatState.activeChatId) return chatState.activeChatId;
  const chat = await createChat(profileName);
  chatState.activeChatId = chat.id;
  await refreshChats();
  return chat.id;
}

async function step(direction: number): Promise<void> {
  const list = chatState.chats;
  if (list.length === 0) {
    await newChat();
    return;
  }
  const current = list.findIndex((chat) => chat.id === chatState.activeChatId);
  if (current === -1) {
    // On the blank chat: Up opens the newest; Down stays put.
    if (direction < 0) await openChat(list[list.length - 1].id);
    return;
  }

  const nextIndex = current + direction;
  if (nextIndex < 0) return;
  if (nextIndex >= list.length) {
    await newChat();
    return;
  }
  await openChat(list[nextIndex].id);
}
