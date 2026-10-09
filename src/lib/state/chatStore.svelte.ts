import type {
  Attachment,
  Chat,
  SendRequest,
  Source,
  ToolStep,
  Usage,
  WebOverride,
  WebSearchMode,
} from "../types";

import { nextWebOverride } from "../webSearch";

export type ChatMessage = {
  id?: string;
  role: "user" | "assistant";
  content: string;
  error?: string;
  model?: string;
  usage?: Usage;
  attachments?: Attachment[];
  steps?: ToolStep[];
  sources?: Source[];
};

export type SendOptions = { profileName: string } & Omit<SendRequest, "chatId" | "messages">;

export const chatState = $state<{
  chats: Chat[];
  activeChatId: string | null;
  messages: ChatMessage[];
  streaming: boolean;
  streamingChatId: string | null;
  sending: boolean;
  thinking: boolean;
  usage: Usage | null;
  webOverrides: Record<string, WebOverride>;
  webDraft: WebOverride | null;
}>({
  chats: [],
  activeChatId: null,
  messages: [],
  streaming: false,
  streamingChatId: null,
  sending: false,
  thinking: false,
  usage: null,
  webOverrides: {},
  webDraft: null,
});

// null means the chat follows its profile's webSearch mode.
export function webOverride(): WebOverride | null {
  const id = chatState.activeChatId;
  return id ? (chatState.webOverrides[id] ?? null) : chatState.webDraft;
}

// Cycle off → auto → deep, based on the mode currently in effect.
export function cycleWeb(current: WebSearchMode): void {
  const next = nextWebOverride(current);
  const id = chatState.activeChatId;
  if (id) chatState.webOverrides[id] = next;
  else chatState.webDraft = next;
}

export function adoptWebDraft(chatId: string): void {
  if (chatState.webDraft === null) return;
  chatState.webOverrides[chatId] = chatState.webDraft;
  chatState.webDraft = null;
}

export function chats(): Chat[] {
  return chatState.chats;
}

export function activeChatId(): string | null {
  return chatState.activeChatId;
}

export function messages(): ChatMessage[] {
  return chatState.messages;
}

export function isStreaming(): boolean {
  return chatState.streaming;
}

export function isStreamingHere(): boolean {
  return chatState.streaming && chatState.streamingChatId === chatState.activeChatId;
}

export function isThinking(): boolean {
  return chatState.thinking && isStreamingHere();
}

export function isBlank(): boolean {
  return chatState.activeChatId === null && chatState.messages.length === 0;
}

export function lastUsage(): Usage | null {
  return chatState.usage;
}

export function lastUserMessage(): ChatMessage | undefined {
  for (let index = chatState.messages.length - 1; index >= 0; index -= 1) {
    if (chatState.messages[index].role === "user") return chatState.messages[index];
  }
  return undefined;
}
