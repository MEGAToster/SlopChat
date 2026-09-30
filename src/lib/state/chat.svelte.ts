import { Channel } from "@tauri-apps/api/core";

import { send as sendRequest, stop as stopStream } from "../ipc";
import type { StreamEvent, Usage } from "../types";

export type ChatMessage = {
  role: "user" | "assistant";
  content: string;
  usage?: Usage;
};

export type SendOptions = {
  model: string;
  system?: string;
  temperature?: number;
};

const CHAT_ID = "current";

const state = $state<{
  messages: ChatMessage[];
  streaming: boolean;
  usage: Usage | null;
  error: string | null;
}>({ messages: [], streaming: false, usage: null, error: null });

export function messages(): ChatMessage[] {
  return state.messages;
}

export function isStreaming(): boolean {
  return state.streaming;
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

export async function submit(text: string, options: SendOptions): Promise<void> {
  const content = text.trim();
  if (state.streaming || !content) return;

  state.error = null;
  state.messages.push({ role: "user", content });
  state.messages.push({ role: "assistant", content: "" });
  const assistantIndex = state.messages.length - 1;
  state.streaming = true;

  const history = state.messages
    .slice(0, assistantIndex)
    .map((message) => ({ role: message.role, content: message.content }));

  const channel = new Channel<StreamEvent>();
  channel.onmessage = (event) => {
    switch (event.type) {
      case "delta":
        state.messages[assistantIndex].content += event.content;
        break;
      case "usage": {
        const usage: Usage = {
          tokensIn: event.tokensIn,
          tokensOut: event.tokensOut,
          cost: event.cost,
        };
        state.messages[assistantIndex].usage = usage;
        state.usage = usage;
        break;
      }
      case "done":
        state.streaming = false;
        break;
      case "error":
        state.error = event.message;
        state.streaming = false;
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
  } finally {
    state.streaming = false;
  }
}

export async function stop(): Promise<void> {
  await stopStream(CHAT_ID);
}
