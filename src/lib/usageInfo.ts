import type { Usage } from "./types";

export function compact(n: number): string {
  return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n);
}

export function money(value: number): string {
  return `$${value.toFixed(4)}`;
}

// The upstream host comes only from the provider; Ollama is labelled "local"
// because its replies never leave the machine.
export function providerLabel(usage: Usage, apiProvider: string): string {
  if (usage.provider) return usage.provider;
  if (apiProvider === "ollama") return "local";
  if (apiProvider === "antigravity") return "antigravity";
  return "";
}

export function usageLine(usage: Usage, apiProvider: string): string {
  const parts = [`↑${compact(usage.tokensIn)} ↓${compact(usage.tokensOut)}`];
  if (usage.cost !== null) parts.push(money(usage.cost));
  const host = providerLabel(usage, apiProvider);
  if (host) parts.push(host);
  return parts.join(" · ");
}
