import { Channel } from "@tauri-apps/api/core";

import {
  appendMessage,
  createChat,
  deleteChat as deleteChatCommand,
  getChat,
  initialChat,
  listChats,
  rewindTo,
  send as sendRequest,
  stop as stopStream,
  toggleChatPin,
} from "../ipc";
import type { Chat, StoredMessage, StreamEvent, Usage } from "../types";

export type ChatMessage = {
  id?: string;
  role: "user" | "assistant";
  content: string;
  usage?: Usage;
};

export type SendOptions = {
  profileName: string;
  model: string;
  system?: string;
  temperature?: number;
};

const CHAT_ID = "current";
let currentProfile = "default";

const state = $state<{
  chats: Chat[];
  activeChatId: string | null;
  messages: ChatMessage[];
  streaming: boolean;
  usage: Usage | null;
  error: string | null;
}>({
  chats: [],
  activeChatId: null,
  messages: [],
  streaming: false,
  usage: null,
  error: null,
});

export function chats(): Chat[] {
  return state.chats;
}

export function activeChatId(): string | null {
  return state.activeChatId;
}

export function messages(): ChatMessage[] {
  return state.messages;
}

export function isStreaming(): boolean {
  return state.streaming;
}

export function activeChatIsEmpty(): boolean {
  return state.messages.length === 0;
}

export function lastUsage(): Usage | null {
  return state.usage;
}

export function chatError(): string | null {
  return state.error;
}

export function reportError(message: string): void {
  state.error = message;
}

export function lastUserMessage(): ChatMessage | undefined {
  for (let index = state.messages.length - 1; index >= 0; index -= 1) {
    if (state.messages[index].role === "user") return state.messages[index];
  }
  return undefined;
}

export function setCurrentProfile(name: string): void {
  if (name) currentProfile = name;
}

export async function init(): Promise<void> {
  const view = await initialChat();
  if (view) {
    state.activeChatId = view.chat.id;
    state.messages = view.messages.map(toChatMessage);
    state.usage = lastUsageOf(view.messages);
  }
  await refreshChats();
  if (state.chats.length === 0) await newChat();
}

export async function refreshChats(): Promise<void> {
  state.chats = await listChats();
}

export async function openChat(chatId: string): Promise<void> {
  const view = await getChat(chatId);
  if (!view) return;
  state.activeChatId = view.chat.id;
  state.messages = view.messages.map(toChatMessage);
  state.usage = lastUsageOf(view.messages);
  state.error = null;
  await refreshChats();
}

export async function newChat(): Promise<void> {
  const chat = await createChat(currentProfile);
  state.activeChatId = chat.id;
  state.messages = [];
  state.usage = null;
  state.error = null;
  await refreshChats();
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
  const wasActive = state.activeChatId === chatId;
  await deleteChatCommand(chatId);
  await refreshChats();

  if (!wasActive) return;

  if (state.chats.length === 0) {
    state.activeChatId = null;
    state.messages = [];
    state.usage = null;
    state.error = null;
    return;
  }

  await openChat(state.chats[state.chats.length - 1].id);
}

export async function submit(text: string, options: SendOptions): Promise<void> {
  const content = text.trim();
  if (state.streaming || !content) return;

  await ensureChat(options.profileName);
  if (!state.activeChatId) return;

  const stored = await appendMessage({ chatId: state.activeChatId, role: "user", content });
  state.messages.push(toChatMessage(stored));
  await refreshChats();

  await streamReply(options);
}

export async function regenerate(
  messageId: string,
  text: string,
  options: SendOptions,
): Promise<void> {
  const content = text.trim();
  if (state.streaming || !state.activeChatId || !content) return;

  await rewindTo(state.activeChatId, messageId, content);
  const view = await getChat(state.activeChatId);
  state.messages = view ? view.messages.map(toChatMessage) : [];
  await refreshChats();

  await streamReply(options);
}

export async function stop(): Promise<void> {
  await stopStream(CHAT_ID);
}

async function ensureChat(profileName: string): Promise<void> {
  if (state.activeChatId) return;
  const chat = await createChat(profileName);
  state.activeChatId = chat.id;
  await refreshChats();
}

async function step(direction: number): Promise<void> {
  const list = state.chats;
  if (list.length === 0) {
    await newChat();
    return;
  }
  const current = list.findIndex((chat) => chat.id === state.activeChatId);
  const nextIndex = current === -1 ? list.length - 1 : current + direction;
  if (nextIndex < 0) return;
  if (nextIndex >= list.length) {
    if (state.messages.length > 0) await newChat();
    return;
  }
  await openChat(list[nextIndex].id);
}

async function streamReply(options: SendOptions): Promise<void> {
  state.error = null;
  state.messages.push({ role: "assistant", content: "" });
  const index = state.messages.length - 1;
  state.streaming = true;

  const history = state.messages
    .slice(0, index)
    .map((message) => ({ role: message.role, content: message.content }));

  let usage: Usage | undefined;
  const channel = new Channel<StreamEvent>();
  channel.onmessage = (event) => {
    switch (event.type) {
      case "delta":
        state.messages[index].content += event.content;
        break;
      case "usage":
        usage = { tokensIn: event.tokensIn, tokensOut: event.tokensOut, cost: event.cost };
        state.messages[index].usage = usage;
        state.usage = usage;
        break;
      case "done":
        state.streaming = false;
        void finalize(index, usage, options.model);
        break;
      case "error":
        state.error = event.message;
        state.streaming = false;
        void finalize(index, usage, options.model);
        break;
    }
  };

  try {
    await sendRequest(
      {
        chatId: CHAT_ID,
        model: options.model,
        system: options.system,
        temperature: options.temperature,
        messages: history,
      },
      channel,
    );
  } catch (error) {
    state.error = String(error);
    state.streaming = false;
    if (!state.messages[index].content) state.messages.splice(index, 1);
  }
}

async function finalize(index: number, usage: Usage | undefined, model: string): Promise<void> {
  const message = state.messages[index];
  if (!message || !message.content || !state.activeChatId) return;

  const stored = await appendMessage({
    chatId: state.activeChatId,
    role: "assistant",
    content: message.content,
    model,
    tokensIn: usage?.tokensIn,
    tokensOut: usage?.tokensOut,
    cost: usage?.cost ?? undefined,
  });
  message.id = stored.id;
  await refreshChats();
}

function toChatMessage(stored: StoredMessage): ChatMessage {
  const hasUsage =
    stored.tokensIn !== null || stored.tokensOut !== null || stored.cost !== null;
  return {
    id: stored.id,
    role: stored.role === "user" ? "user" : "assistant",
    content: stored.content,
    usage: hasUsage
      ? {
          tokensIn: stored.tokensIn ?? 0,
          tokensOut: stored.tokensOut ?? 0,
          cost: stored.cost,
        }
      : undefined,
  };
}

function lastUsageOf(messages: StoredMessage[]): Usage | null {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message.tokensIn !== null || message.tokensOut !== null || message.cost !== null) {
      return {
        tokensIn: message.tokensIn ?? 0,
        tokensOut: message.tokensOut ?? 0,
        cost: message.cost,
      };
    }
  }
  return null;
}
