import { splitModelRef } from "./modelRef";
import { reasoningInfo, reasoningOverride } from "./state/reasoning.svelte";
import { webOverride } from "./state/chatStore.svelte";
import { effectiveWebMode } from "./webSearch";
import type { ModelConfig, WebSearchMode } from "./types";

export function isAntigravity(model: string | null | undefined): boolean {
  return splitModelRef(model ?? "").provider === "antigravity";
}

// Shared by the footer and the expanded empty state.
export function reasoningEffort(config: ModelConfig | null | undefined): string {
  return (
    reasoningOverride() ?? config?.reasoning ?? reasoningInfo(config?.model ?? "")?.defaultEffort ?? "auto"
  );
}

export function webMode(config: ModelConfig | null | undefined): WebSearchMode {
  if (isAntigravity(config?.model)) return "off";
  return effectiveWebMode(config?.toolCalling !== false, webOverride(), config?.webSearch);
}
