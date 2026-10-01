import { Channel } from "@tauri-apps/api/core";

import { send as sendRequest, stop as stopStream } from "../ipc";
import { stripControlTokens } from "../markdown/clean";
import type { RequestMessage, Source, StreamEvent, ToolStep, Usage } from "../types";

export type StreamRequest = {
  model: string;
  system?: string;
  temperature?: number;
  useContext?: boolean;
  tools?: string[];
  toolCalling?: boolean;
  webSearch?: "auto" | "always" | "off";
  webToggle?: boolean;
  messages: RequestMessage[];
};

export type StreamCallbacks = {
  onUpdate: (content: string) => void;
  onReasoning: () => void;
  onUsage: (usage: Usage) => void;
  onSteps: (steps: ToolStep[]) => void;
  onSources: (sources: Source[]) => void;
};

export type StreamOutcome = {
  content: string;
  usage?: Usage;
  error?: string;
  reasoning: boolean;
  steps: ToolStep[];
  sources: Source[];
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
  const steps: ToolStep[] = [];
  const sources: Source[] = [];

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
      case "toolCall":
        steps.push({
          id: event.id,
          name: event.name,
          args: event.args,
          status: "running",
          summary: "",
        });
        callbacks.onSteps([...steps]);
        break;
      case "toolResult": {
        const step = steps.find((entry) => entry.id === event.id);
        if (step) {
          step.status = event.ok ? "done" : "error";
          step.summary = event.summary;
        }
        sources.push(...event.sources);
        callbacks.onSteps([...steps]);
        callbacks.onSources([...sources]);
        break;
      }
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

  return { content: stripControlTokens(content), usage, error, reasoning, steps, sources };
}

export async function stop(chatId: string): Promise<void> {
  await stopStream(chatId);
}
