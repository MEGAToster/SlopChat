// OpenRouter routing variants appended to a base model id.
export const VARIANT_PATTERN = /:(floor|nitro|online|free|thinking)$/;
export const MODEL_TAGS = ["", ":floor", ":nitro", ":online", ":free", ":thinking"];

export type ModelRefParts = { provider: string; modelId: string; tag: string };

export function splitModelRef(ref: string): ModelRefParts {
  const slash = ref.indexOf("/");
  const provider = slash === -1 ? ref : ref.slice(0, slash);
  let rest = slash === -1 ? "" : ref.slice(slash + 1);
  const match = rest.match(VARIANT_PATTERN);
  const tag = match ? match[0] : "";
  if (tag) rest = rest.slice(0, -tag.length);
  return { provider, modelId: rest, tag };
}

export function composeModelRef(provider: string, modelId: string, tag: string): string {
  const base = modelId.trim().replace(/^\/+/, "");
  if (!base) return "";
  const prefixed = base.startsWith(`${provider}/`) ? base : `${provider}/${base}`;
  return `${prefixed}${tag}`;
}
