import { Channel } from "@tauri-apps/api/core";

import { send as sendRequest, stop as stopStream } from "../ipc";
import { stripControlTokens } from "../markdown/clean";
import type { SendRequest, Source, StreamEvent, ToolStep, Usage } from "../types";

type StreamRequest = Omit<SendRequest, "chatId">;

type StreamCallbacks = {
  onUpdate: (content: string) => void;
  onThinking: (active: boolean) => void;
  onUsage: (usage: Usage) => void;
  onSteps: (steps: ToolStep[]) => void;
  onSources: (sources: Source[]) => void;
};

type StreamOutcome = {
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
  // Text is buffered per round: a round that ends in tool calls is narration,
  // the final round is the answer.
  let answer = "";
  let round = -1;
  let notePending = false;
  let usage: Usage | undefined;
  let error: string | undefined;
  let reasoning = false;
  const steps: ToolStep[] = [];
  const sources: Source[] = [];

  const channel = new Channel<StreamEvent>();
  channel.onmessage = (event) => {
    switch (event.type) {
      case "delta":
        callbacks.onThinking(false);
        if (event.round !== round) {
          round = event.round;
          answer = "";
          notePending = true;
        }
        answer += event.content;
        callbacks.onUpdate(answer);
        break;
      case "reasoning":
        reasoning = true;
        callbacks.onThinking(true);
        break;
      case "toolCall": {
        callbacks.onThinking(false);
        const note = notePending && answer.trim() ? answer.trim() : undefined;
        if (note) {
          answer = "";
          callbacks.onUpdate(answer);
        }
        notePending = false;
        steps.push({
          id: event.id,
          name: event.name,
          args: event.args,
          status: "running",
          summary: "",
          note,
        });
        callbacks.onSteps([...steps]);
        break;
      }
      case "toolResult": {
        const step = steps.find((entry) => entry.id === event.id);
        if (step) {
          step.status = event.skipped ? "skipped" : event.ok ? "done" : "error";
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

  return { content: stripControlTokens(answer), usage, error, reasoning, steps, sources };
}

export async function stop(chatId: string): Promise<void> {
  await stopStream(chatId);
}
