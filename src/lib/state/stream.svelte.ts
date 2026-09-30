import { Channel } from "@tauri-apps/api/core";

import { send as sendRequest, stop as stopStream } from "../ipc";
import type { RequestMessage, StreamEvent, Usage } from "../types";

export type StreamRequest = {
  model: string;
  system?: string;
  temperature?: number;
  useContext?: boolean;
  messages: RequestMessage[];
};

export type StreamCallbacks = {
  onUpdate: (content: string) => void;
  onReasoning: () => void;
  onUsage: (usage: Usage) => void;
};

export type StreamOutcome = {
  content: string;
  usage?: Usage;
  error?: string;
  reasoning: boolean;
};

export async function run(
  chatId: string,
  request: StreamRequest,
  callbacks: StreamCallbacks,
): Promise<StreamOutcome> {
  let content = "";
  let usage: Usage | undefined;
  let error: string | undefined;
  let reasoning = false;

  const channel = new Channel<StreamEvent>();
  channel.onmessage = (event) => {
    switch (event.type) {
      case "delta":
        content += event.content;
        callbacks.onUpdate(content);
        break;
      case "reasoning":
        reasoning = true;
        callbacks.onReasoning();
        break;
      case "usage":
        usage = {
          tokensIn: event.tokensIn,
          tokensOut: event.tokensOut,
          cost: event.cost,
          provider: event.provider,
        };
        callbacks.onUsage(usage);
        break;
      case "error":
        error = event.message;
        break;
    }
  };

  try {
    await sendRequest({ chatId, ...request }, channel);
  } catch (cause) {
    error = String(cause);
  }

  return { content, usage, error, reasoning };
}

export async function stop(chatId: string): Promise<void> {
  await stopStream(chatId);
}
