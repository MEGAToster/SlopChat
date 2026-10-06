import type { SearchSettings, WebOverride, WebSearchMode } from "./types";

// The mode actually sent: the chat override wins, otherwise the model/profile
// default. Tool calling off means no web at all.
export function effectiveWebMode(
  toolCalling: boolean,
  override: WebOverride | null,
  configured?: WebSearchMode,
): WebSearchMode {
  if (!toolCalling) return "off";
  return override ?? configured ?? "auto";
}

// The override Ctrl+W sets next: off → auto → deep → off.
export function nextWebOverride(current: WebSearchMode): WebOverride {
  if (current === "off") return "auto";
  if (current === "auto") return "deep";
  return "off";
}

// Deep research is meant to run on free SearXNG; return a warning when it would
// fall back to a paid provider, else null.
export function paidSearchWarning(search: SearchSettings): string | null {
  const providers = search.providers ?? [];
  const searxngReady =
    (search.searxng?.baseUrl ?? "").trim() !== "" && providers[0] === "searxng";
  if (searxngReady) return null;
  const paid = providers.filter((name) => name !== "searxng").join(", ");
  return `Deep research will use paid search (${paid || "a paid provider"}) — set up SearXNG in Settings → Web search.`;
}
