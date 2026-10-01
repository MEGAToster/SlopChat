<script lang="ts">
  import { untrack } from "svelte";

  import type { ModelConfig, Profile, ProfileOverrides, WebSearchMode } from "../lib/types";
  import { REASONING_LEVELS } from "../lib/reasoningLevels";

  let {
    models,
    value,
    previous,
    onsave,
    oncancel,
    onremove,
  }: {
    models: ModelConfig[];
    value: Profile;
    previous: string | null;
    onsave: (profile: Profile, previous: string | null) => void;
    oncancel: () => void;
    onremove?: () => void;
  } = $props();

  const COLORS = ["", "purple", "blue", "green", "orange", "pink", "cyan", "yellow", "red"];
  const INHERIT = "(inherit)";

  const defaultModel = untrack(() => models[0]?.model ?? "");
  const start = untrack(() => $state.snapshot(value));

  let name = $state(start.name);
  let model = $state(start.model || defaultModel);
  let color = $state(start.color ?? "");
  let system = $state(start.system ?? "");
  let temperature = $state(start.temperature !== undefined ? String(start.temperature) : "");
  let useContext = $state(start.useContext === undefined ? "" : start.useContext ? "on" : "off");
  let tools = $state(start.tools === undefined ? "" : start.tools.length > 0 ? "web" : "none");
  let toolCalling = $state(
    start.toolCalling === undefined ? "" : start.toolCalling ? "on" : "off",
  );
  let webSearch = $state(start.webSearch ?? "");
  let reasoning = $state(start.reasoning ?? "");
  let fallback = $state(start.fallback ?? "");

  function save(): void {
    if (!name.trim() || !model) return;
    const overrides: ProfileOverrides = {};
    if (system.trim()) overrides.system = system.trim();
    if (temperature.trim()) overrides.temperature = Number(temperature);
    if (useContext) overrides.useContext = useContext === "on";
    if (tools) overrides.tools = tools === "web" ? ["web_search", "web_fetch"] : [];
    if (toolCalling) overrides.toolCalling = toolCalling === "on";
    if (webSearch) overrides.webSearch = webSearch as WebSearchMode;
    if (reasoning) overrides.reasoning = reasoning;

    onsave(
      {
        name: name.trim(),
        model,
        fallback: fallback || undefined,
        color: color || undefined,
        ...overrides,
      },
      previous,
    );
  }
</script>

<div class="body">
  <div class="section-title">{previous ? "Edit rotation entry" : "Add to rotation"}</div>

  <label class="row">
    <span class="label">Name</span>
    <input class="control" bind:value={name} placeholder="profile name" />
  </label>

  <label class="row">
    <span class="label">Model</span>
    <select class="control" bind:value={model}>
      {#each models as entry (entry.model)}<option value={entry.model}>{entry.model}</option>{/each}
    </select>
  </label>

  <details class="fallback">
    <summary>
      <span class="label">Fallback model</span>
      <span class="value">{fallback || "none"}</span>
    </summary>
    <label class="row">
      <span class="label">If unavailable</span>
      <select class="control" bind:value={fallback}>
        <option value="">none</option>
        {#each models as entry (entry.model)}
          <option value={entry.model}>{entry.model}</option>
        {/each}
      </select>
    </label>
  </details>

  <label class="row">
    <span class="label">Color</span>
    <select class="control" bind:value={color}>
      {#each COLORS as option (option)}<option value={option}>{option || "auto"}</option>{/each}
    </select>
  </label>

  <label class="row">
    <span class="label">System prompt</span>
    <input class="control" bind:value={system} placeholder={INHERIT} />
  </label>

  <label class="row">
    <span class="label">Temperature</span>
    <input class="control" bind:value={temperature} placeholder={INHERIT} />
  </label>

  <label class="row">
    <span class="label">Use context</span>
    <select class="control" bind:value={useContext}>
      <option value="">{INHERIT}</option>
      <option value="on">on</option>
      <option value="off">off</option>
    </select>
  </label>

  <label class="row">
    <span class="label">Tool calling</span>
    <select class="control" bind:value={toolCalling}>
      <option value="">{INHERIT}</option>
      <option value="on">on</option>
      <option value="off">off</option>
    </select>
  </label>

  <label class="row">
    <span class="label">Tools</span>
    <select class="control" bind:value={tools}>
      <option value="">{INHERIT}</option>
      <option value="none">none</option>
      <option value="web">web search + fetch</option>
    </select>
  </label>

  <label class="row">
    <span class="label">Web search</span>
    <select class="control" bind:value={webSearch}>
      <option value="">{INHERIT}</option>
      <option value="auto">auto</option>
      <option value="always">always</option>
      <option value="off">off</option>
    </select>
  </label>

  <label class="row">
    <span class="label">Reasoning</span>
    <select class="control" bind:value={reasoning}>
      <option value="">{INHERIT}</option>
      {#each REASONING_LEVELS.filter(Boolean) as level (level)}
        <option value={level}>{level}</option>
      {/each}
    </select>
  </label>

  <div class="actions">
    {#if onremove}
      <button type="button" class="danger" onclick={onremove}>Delete</button>
    {/if}
    <span class="spacer"></span>
    <button type="button" class="ghost" onclick={oncancel}>Cancel</button>
    <button type="button" class="primary" disabled={!name.trim() || !model} onclick={save}>Save</button>
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

  .control:focus {
    border-color: var(--accent);
  }

  .fallback > summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    min-height: 40px;
    padding: 0 var(--sp-2);
    cursor: default;
    list-style: none;
  }

  .fallback > summary::-webkit-details-marker {
    display: none;
  }

  .value {
    overflow: hidden;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
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
