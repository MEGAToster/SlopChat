import {
  appendMessage,
  createChat,
  deleteChat as deleteChatCommand,
  getChat,
  initialChat,
  listChats,
  rewindTo,
  toggleChatPin,
} from "../ipc";
import type { Chat, Usage } from "../types";
import { lastUsageOf, toChatMessage } from "./messages";
import { run as runStream, stop as stopStream } from "./stream.svelte";

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
  useContext?: boolean;
};

const state = $state<{
  chats: Chat[];
  activeChatId: string | null;
  messages: ChatMessage[];
  streaming: boolean;
  streamingChatId: string | null;
  thinking: boolean;
  usage: Usage | null;
  error: string | null;
}>({
  chats: [],
  activeChatId: null,
  messages: [],
  streaming: false,
  streamingChatId: null,
  thinking: false,
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

export function isStreamingHere(): boolean {
  return state.streaming && state.streamingChatId === state.activeChatId;
}

export function isThinking(): boolean {
  return state.thinking && isStreamingHere();
}

export function isBlank(): boolean {
  return state.activeChatId === null && state.messages.length === 0;
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

export async function init(): Promise<void> {
  const view = await initialChat();
  if (view) {
    state.activeChatId = view.chat.id;
    state.messages = view.messages.map(toChatMessage);
    state.usage = lastUsageOf(view.messages);
  }
  await refreshChats();
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
  if (state.activeChatId === null && state.messages.length === 0) return;
  state.activeChatId = null;
  state.messages = [];
  state.usage = null;
  state.error = null;
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
    await newChat();
    return;
  }

  await openChat(state.chats[state.chats.length - 1].id);
}

export async function submit(text: string, options: SendOptions): Promise<void> {
  const content = text.trim();
  if (state.streaming || !content) return;

  const chatId = await ensureChat(options.profileName);
  const stored = await appendMessage({ chatId, role: "user", content });
  state.messages.push(toChatMessage(stored));
  await refreshChats();

  await streamReply(chatId, options);
}

export async function regenerate(
  messageId: string,
  text: string,
  options: SendOptions,
): Promise<void> {
  const content = text.trim();
  if (state.streaming || !state.activeChatId || !content) return;

  const chatId = state.activeChatId;
  await rewindTo(chatId, messageId, content);
  const view = await getChat(chatId);
  state.messages = view ? view.messages.map(toChatMessage) : [];
  await refreshChats();

  await streamReply(chatId, options);
}

export async function stop(): Promise<void> {
  if (state.streamingChatId) await stopStream(state.streamingChatId);
}

async function ensureChat(profileName: string): Promise<string> {
  if (state.activeChatId) return state.activeChatId;
  const chat = await createChat(profileName);
  state.activeChatId = chat.id;
  await refreshChats();
  return chat.id;
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
    await newChat();
    return;
  }
  await openChat(list[nextIndex].id);
}

async function streamReply(chatId: string, options: SendOptions): Promise<void> {
  state.error = null;

  const tempId = `pending-${Date.now()}`;
  state.messages.push({ id: tempId, role: "assistant", content: "" });
  const index = state.messages.length - 1;
  state.streaming = true;
  state.streamingChatId = chatId;
  state.thinking = false;

  const history = state.messages
    .slice(0, index)
    .map((message) => ({ role: message.role, content: message.content }));

  const find = () => state.messages.find((message) => message.id === tempId);

  const outcome = await runStream(
    chatId,
    {
      model: options.model,
      system: options.system,
      temperature: options.temperature,
      useContext: options.useContext,
      messages: history,
    },
    {
      onUpdate: (content) => {
        if (state.activeChatId !== chatId) return;
        const message = find();
        if (message) message.content = content;
      },
      onReasoning: () => {
        state.thinking = true;
      },
      onUsage: (usage) => {
        if (state.activeChatId !== chatId) return;
        const message = find();
        if (message) message.usage = usage;
      },
    },
  );

  state.streaming = false;
  state.streamingChatId = null;
  state.thinking = false;

  if (outcome.error) state.error = outcome.error;

  if (outcome.content) {
    const stored = await appendMessage({
      chatId,
      role: "assistant",
      content: outcome.content,
      model: options.model,
      tokensIn: outcome.usage?.tokensIn,
      tokensOut: outcome.usage?.tokensOut,
      cost: outcome.usage?.cost ?? undefined,
    });
    const message = find();
    if (message) message.id = stored.id;
    if (state.activeChatId === chatId && outcome.usage) state.usage = outcome.usage;
    await refreshChats();
  } else {
    const emptyIndex = state.messages.findIndex((message) => message.id === tempId);
    if (emptyIndex !== -1) state.messages.splice(emptyIndex, 1);
  }
}


