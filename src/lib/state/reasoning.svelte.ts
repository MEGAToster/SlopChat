import { modelReasoning } from "../ipc";
import type { ReasoningInfo } from "../types";
import { chatState } from "./chatStore.svelte";

// null means "use the model's configured default".
type Override = string | null;

const state = $state({
  overrides: {} as Record<string, Override>,
  draft: null as Override,
  info: {} as Record<string, ReasoningInfo | null>,
});

export function reasoningOverride(): Override {
  const id = chatState.activeChatId;
  return id ? (state.overrides[id] ?? null) : state.draft;
}

export function reasoningInfo(model: string): ReasoningInfo | null | undefined {
  return state.info[model];
}

export async function loadReasoning(model: string): Promise<void> {
  if (!model || model in state.info) return;
  state.info[model] = await modelReasoning(model).catch(() => null);
}

function cycleReasoning(model: string): void {
  const info = state.info[model];
  const efforts = info?.supportedEfforts ?? [];
  const levels: Override[] = [null, ...efforts];
  const next = levels[(levels.indexOf(reasoningOverride()) + 1) % levels.length] ?? null;

  const id = chatState.activeChatId;
  if (id) state.overrides[id] = next;
  else state.draft = next;
}

export async function cycleReasoningFor(model: string): Promise<void> {
  await loadReasoning(model);
  cycleReasoning(model);
}

export function adoptReasoningDraft(chatId: string): void {
  if (state.draft === null) return;
  state.overrides[chatId] = state.draft;
  state.draft = null;
}
