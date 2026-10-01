import { addModel, removeModel, resizePanel } from "../ipc";
import { isExpanded, OVERLAY_HEIGHT } from "./ui.svelte";

const VARIANTS = ["", ":floor", ":nitro", ":online", ":free", ":thinking"];
const VARIANT_PATTERN = /:(floor|nitro|online|free|thinking)$/;
const PROVIDERS_WITH_VARIANTS = new Set(["openrouter"]);

const state = $state({
  open: false,
  provider: "openrouter",
  modelId: "",
  tag: "",
});

export function isOpen(): boolean {
  return state.open;
}

export function provider(): string {
  return state.provider;
}

export function modelId(): string {
  return state.modelId;
}

export function tag(): string {
  return state.tag;
}

export function tags(): string[] {
  return PROVIDERS_WITH_VARIANTS.has(state.provider) ? VARIANTS : [""];
}

export function setProvider(value: string): void {
  state.provider = value;
  if (!tags().includes(state.tag)) state.tag = "";
}

export function setModelId(value: string): void {
  state.modelId = value;
}

export function setTag(value: string): void {
  state.tag = value;
}

export function openPanel(prefill?: { provider?: string; modelId?: string; tag?: string }): void {
  state.provider = prefill?.provider ?? "openrouter";
  state.modelId = prefill?.modelId ?? "";
  state.tag = prefill?.tag ?? "";
  state.open = true;
  if (!isExpanded()) void resizePanel(OVERLAY_HEIGHT);
}

export function openFromProvider(provider: string, modelRef: string): void {
  let id = modelRef;
  let tag = "";

  const match = id.match(VARIANT_PATTERN);
  if (match) {
    tag = match[0];
    id = id.slice(0, -tag.length);
  }
  if (id.startsWith(`${provider}/`)) id = id.slice(provider.length + 1);

  openPanel({ provider, modelId: id, tag });
}

export function closePanel(): void {
  state.open = false;
}

export function composedModel(): string {
  const base = state.modelId
    .trim()
    .replace(VARIANT_PATTERN, "")
    .replace(/^\/+/, "");
  if (!base) return "";

  const prefixed = base.startsWith(`${state.provider}/`) ? base : `${state.provider}/${base}`;
  return `${prefixed}${state.tag}`;
}

export async function add(): Promise<void> {
  const model = composedModel();
  if (!model) return;
  await addModel(model);
  state.modelId = "";
  state.tag = "";
}

export async function remove(model: string): Promise<void> {
  await removeModel(model);
}
