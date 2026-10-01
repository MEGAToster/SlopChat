import { resizePanel } from "../ipc";
import { composeModelRef, splitModelRef } from "../modelRef";
import { isExpanded, OVERLAY_HEIGHT } from "./ui.svelte";

const state = $state({
  open: false,
  provider: "openrouter",
  modelId: "",
  tag: "",
});

export function isOpen(): boolean {
  return state.open;
}

export function modelId(): string {
  return state.modelId;
}

export function openPanel(prefill?: { provider?: string; modelId?: string; tag?: string }): void {
  state.provider = prefill?.provider ?? "openrouter";
  state.modelId = prefill?.modelId ?? "";
  state.tag = prefill?.tag ?? "";
  state.open = true;
  if (!isExpanded()) void resizePanel(OVERLAY_HEIGHT);
}

export function openFromProvider(provider: string, modelRef: string): void {
  const { modelId, tag } = splitModelRef(modelRef);
  openPanel({ provider, modelId, tag });
}

export function closePanel(): void {
  state.open = false;
}

// The palette's "Add models…" flow pre-fills this; Settings opens the editor
// with the composed reference.
export function composedModel(): string {
  return composeModelRef(state.provider, state.modelId, state.tag);
}
