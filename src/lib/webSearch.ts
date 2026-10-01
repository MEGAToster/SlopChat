import type { WebOverride, WebSearchMode } from "./types";

// The mode actually sent: profile/mode default, unless the chat override or
// tool-calling support changes it.
export function effectiveWebMode(
  toolCalling: boolean,
  override: WebOverride | null,
  configured?: WebSearchMode,
): WebSearchMode {
  if (!toolCalling || override === "off") return "off";
  if (override === "on") return "always";
  return configured ?? "auto";
}
