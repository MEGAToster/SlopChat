import type { Chat, Usage } from "../types";

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

export const chatState = $state<{
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

export function chatError(): string | null {
  return chatState.error;
}

export function reportError(message: string): void {
  chatState.error = message;
}

export function lastUserMessage(): ChatMessage | undefined {
  for (let index = chatState.messages.length - 1; index >= 0; index -= 1) {
    if (chatState.messages[index].role === "user") return chatState.messages[index];
  }
  return undefined;
}
