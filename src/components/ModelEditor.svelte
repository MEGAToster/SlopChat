<script lang="ts">
  import { untrack } from "svelte";

  import type { ModelConfig, WebSearchMode } from "../lib/types";
  import { composeModelRef, MODEL_TAGS, splitModelRef } from "../lib/modelRef";
  import { REASONING_LEVELS } from "../lib/reasoningLevels";

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
    { value: "auto", label: "auto" },
    { value: "always", label: "always" },
    { value: "off", label: "off" },
  ];

  const fallbackProvider = untrack(() => providers[0] ?? "openrouter");
  const start = untrack(() => $state.snapshot(value));
  const parts = splitModelRef(start.model);

  let provider = $state(parts.provider || fallbackProvider);
  let modelId = $state(parts.modelId);
  let tag = $state(parts.tag);
  let reasoning = $state(start.reasoning ?? "");
  let system = $state(start.system ?? "");
  let temperature = $state(start.temperature !== undefined ? String(start.temperature) : "");
  let useContext = $state(start.useContext);
  let toolCalling = $state(start.toolCalling !== false);
  let webSearch = $state<WebSearchMode>(start.webSearch ?? "auto");
  let tools = $state(new Set(start.tools ?? []));

  const composed = $derived(composeModelRef(provider, modelId, tag));

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
        temperature: temperature.trim() ? Number(temperature) : undefined,
        useContext,
        tools: [...tools],
        toolCalling,
        webSearch,
        reasoning: reasoning || undefined,
      },
      previous,
    );
  }
</script>

<div class="body">
  <div class="section-title">{previous ? "Edit model" : "Add model"}</div>

  <label class="row">
    <span class="label">Provider</span>
    <select class="control" bind:value={provider}>
      {#each providers as name (name)}<option value={name}>{name}</option>{/each}
    </select>
  </label>

  <label class="row">
    <span class="label">Model id</span>
    <input class="control" bind:value={modelId} placeholder="deepseek/deepseek-v4.1-flash" />
  </label>

  {#if provider === "openrouter"}
    <label class="row">
      <span class="label">Tag</span>
      <select class="control" bind:value={tag}>
        {#each MODEL_TAGS as option (option)}<option value={option}>{option || "none"}</option>{/each}
      </select>
    </label>
  {/if}

  <div class="row">
    <span class="label">Reference</span>
    <span class="preview">{composed || "—"}</span>
  </div>

  <label class="row">
    <span class="label">Reasoning</span>
    <select class="control" bind:value={reasoning}>
      {#each REASONING_LEVELS as level (level)}
        <option value={level}>{level || "default"}</option>
      {/each}
    </select>
  </label>

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
    <span class="label">Use context</span>
    <span class="check">
      <input type="checkbox" bind:checked={useContext} />
      <span class="box"></span>
    </span>
  </label>

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

  <label class="row block">
    <span class="label">System prompt</span>
    <textarea class="area" bind:value={system} rows="3" placeholder="(none)"></textarea>
  </label>

  <div class="actions">
    {#if onremove}
      <button type="button" class="danger" onclick={onremove}>Delete</button>
    {/if}
    <span class="spacer"></span>
    <button type="button" class="ghost" onclick={oncancel}>Cancel</button>
    <button type="button" class="primary" disabled={!composed} onclick={save}>Save</button>
  </div>
</div>

<style>
  .body {
    display: flex;
    flex-direction: column;
  }

  .section-title {
    height: 28px;
    display: flex;
    align-items: center;
    padding: 0 var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    min-height: 40px;
    padding: 0 var(--sp-2);
  }

  .row.block {
    flex-direction: column;
    align-items: stretch;
    gap: var(--sp-1);
    padding-block: var(--sp-1);
  }

  .label {
    color: var(--text-secondary);
    font-size: var(--fs-md);
  }

  .control {
    width: 60%;
    height: 28px;
    padding: 0 var(--sp-2);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    outline: none;
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-md);
  }

  .control:focus,
  .area:focus {
    border-color: var(--accent);
  }

  .area {
    width: 100%;
    padding: var(--sp-2);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    outline: none;
    resize: vertical;
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
  }

  .preview {
    overflow: hidden;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .checks {
    display: inline-flex;
    gap: var(--sp-3);
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  .check {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
    cursor: default;
  }

  .check input {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
  }

  .check .box {
    position: relative;
    flex: none;
    width: 16px;
    height: 16px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }

  .check input:checked + .box {
    border-color: var(--accent);
    background: var(--accent);
  }

  .check input:checked + .box::after {
    content: "";
    position: absolute;
    left: 5px;
    top: 2px;
    width: 4px;
    height: 8px;
    border: solid var(--accent-contrast);
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .check input:focus-visible + .box {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2);
  }

  .spacer {
    flex: 1;
  }

  .primary,
  .ghost,
  .danger {
    padding: 6px var(--sp-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  .primary {
    border-color: transparent;
    background: var(--accent);
    color: var(--accent-contrast);
    font-weight: 600;
  }

  .primary:disabled {
    opacity: 0.4;
  }

  .danger {
    color: var(--danger);
  }
</style>
