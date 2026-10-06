<script lang="ts">
  import { untrack } from "svelte";

  import type { ModelConfig, Routing, WebSearchMode } from "../lib/types";
  import { composeModelRef, MODEL_TAGS, splitModelRef } from "../lib/modelRef";
  import { hasRouting } from "../lib/routing";
  import { REASONING_LEVELS } from "../lib/reasoningLevels";
  import RoutingEditor from "./RoutingEditor.svelte";

  let {
    providers,
    value,
    previous,
    onsave,
    oncancel,
    onremove,
  }: {
    providers: string[];
    value: ModelConfig;
    previous: string | null;
    onsave: (config: ModelConfig, previous: string | null) => void;
    oncancel: () => void;
    onremove?: () => void;
  } = $props();

  const WEB_MODES: { value: WebSearchMode; label: string }[] = [
    { value: "off", label: "off" },
    { value: "auto", label: "auto" },
    { value: "deep", label: "deep" },
  ];

  const MODEL_PLACEHOLDERS: Record<string, string> = {
    openrouter: "deepseek/deepseek-v4.1-flash",
    ollama: "llama3.2",
    gemini: "gemini-2.5-flash",
    antigravity: "gemini-3.8-flash-medium",
  };

  const HIDDEN_PROVIDERS = new Set(["gemini"]);

  const fallbackProvider = untrack(
    () => (providers.includes("openrouter") ? "openrouter" : providers[0]) ?? "openrouter",
  );
  const start = untrack(() => $state.snapshot(value));
  const parts = splitModelRef(start.model);

  let provider = $state(parts.provider || fallbackProvider);
  let modelId = $state(parts.modelId);
  let tag = $state(parts.tag);
  let reasoning = $state(start.reasoning ?? "");
  let system = $state(start.system ?? "");
  let temperature = $state(start.temperature !== undefined ? String(start.temperature) : "");
  let maxTokens = $state(
    start.maxCompletionTokens !== undefined ? String(start.maxCompletionTokens) : "",
  );
  let useContext = $state(start.useContext);
  let toolCalling = $state(start.toolCalling !== false);
  let webSearch = $state<WebSearchMode>(start.webSearch ?? "auto");
  let tools = $state(new Set(start.tools ?? []));
  let routing = $state<Routing>(start.routing ?? {});

  const composed = $derived(composeModelRef(provider, modelId, tag));
  const isAntigravity = $derived(provider === "antigravity");
  const modelPlaceholder = $derived(MODEL_PLACEHOLDERS[provider] ?? "provider/model");
  const providerOptions = $derived.by(() => {
    const visible = providers.filter((name) => !HIDDEN_PROVIDERS.has(name));
    // Keep an existing model's provider selectable so editing it does not reset.
    return provider && !visible.includes(provider) ? [provider, ...visible] : visible;
  });

  function toggleTool(name: string): void {
    const next = new Set(tools);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    tools = next;
  }

  function save(): void {
    if (!composed) return;
    onsave(
      {
        model: composed,
        system: system.trim() || undefined,
        temperature: isAntigravity ? undefined : temperature.trim() ? Number(temperature) : undefined,
        useContext,
        tools: isAntigravity ? [] : [...tools],
        toolCalling: isAntigravity ? false : toolCalling,
        webSearch: isAntigravity ? undefined : webSearch,
        reasoning: isAntigravity ? undefined : reasoning || undefined,
        maxCompletionTokens: isAntigravity ? undefined : maxTokens.trim() ? Number(maxTokens) : undefined,
        routing: !isAntigravity && hasRouting(routing) ? routing : undefined,
      },
      previous,
    );
  }
</script>

<div class="editor">
  <div class="section-title">{previous ? "Edit model" : "Add model"}</div>

  <label class="row">
    <span class="label">Provider</span>
    <select class="control" bind:value={provider}>
      {#each providerOptions as name (name)}<option value={name}>{name}</option>{/each}
    </select>
  </label>

  <label class="row">
    <span class="label">Model id</span>
    <input class="control" bind:value={modelId} placeholder={modelPlaceholder} />
  </label>

  {#if provider === "openrouter"}
    <label class="row">
      <span class="label">Tag</span>
      <select class="control" bind:value={tag}>
        {#each MODEL_TAGS as option (option)}<option value={option}>{option || "none"}</option>{/each}
      </select>
    </label>
    <RoutingEditor value={routing} onchange={(next) => (routing = next)} />
  {/if}

  <div class="row">
    <span class="label">Reference</span>
    <span class="preview">{composed || "—"}</span>
  </div>

  {#if !isAntigravity}
  <label class="row">
    <span class="label">Reasoning</span>
    <select class="control" bind:value={reasoning}>
      {#each REASONING_LEVELS as level (level)}
        <option value={level}>{level || "default"}</option>
      {/each}
    </select>
  </label>
  {/if}

  {#if !isAntigravity}
  <label class="row">
    <span class="label">Web search</span>
    <select class="control" bind:value={webSearch}>
      {#each WEB_MODES as mode (mode.value)}<option value={mode.value}>{mode.label}</option>{/each}
    </select>
  </label>

  <label class="row">
    <span class="label">Temperature</span>
    <input class="control" bind:value={temperature} placeholder="provider default" />
  </label>

  <label class="row">
    <span class="label">Max completion tokens</span>
    <input class="control" bind:value={maxTokens} placeholder="8192" />
  </label>
  {/if}

  <label class="row">
    <span class="label">Use context</span>
    <span class="check">
      <input type="checkbox" bind:checked={useContext} />
      <span class="box"></span>
    </span>
  </label>

  {#if !isAntigravity}
  <label class="row">
    <span class="label">Tool calling</span>
    <span class="check">
      <input type="checkbox" bind:checked={toolCalling} />
      <span class="box"></span>
    </span>
  </label>

  {#if toolCalling}
    <div class="row">
      <span class="label">Tools</span>
      <span class="checks">
        <label class="check">
          <input type="checkbox" checked={tools.has("web_search")} onchange={() => toggleTool("web_search")} />
          <span class="box"></span>
          web_search
        </label>
        <label class="check">
          <input type="checkbox" checked={tools.has("web_fetch")} onchange={() => toggleTool("web_fetch")} />
          <span class="box"></span>
          web_fetch
        </label>
      </span>
    </div>
  {/if}
  {/if}

  <label class="row block">
    <span class="label">System prompt</span>
    <textarea class="area" bind:value={system} rows="3" placeholder="(none)"></textarea>
  </label>

  <div class="actions">
    {#if onremove}
      <button type="button" class="btn danger" onclick={onremove}>Delete</button>
    {/if}
    <span class="spacer"></span>
    <button type="button" class="btn" onclick={oncancel}>Cancel</button>
    <button type="button" class="btn primary" disabled={!composed} onclick={save}>Save</button>
  </div>
</div>

<style>
  .preview {
    overflow: hidden;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
</style>
