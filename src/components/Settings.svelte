<script lang="ts">
  import { onMount } from "svelte";
  import { removeModel, removeProfile, saveModel, saveProfile, searchUsage } from "../lib/ipc";
  import { REASONING_LEVELS } from "../lib/reasoningLevels";
  import type { SearchProviderUsage } from "../lib/types";
  import {
    add,
    composedModel,
    modelId,
    provider,
    reasoning,
    setModelId,
    setProvider,
    setReasoning,
    setTag,
    tag,
    tags,
  } from "../lib/state/settingsPanel.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import ApiKeyRow from "./ApiKeyRow.svelte";
  import Frame from "./Frame.svelte";
  import ModelRow from "./ModelRow.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const providers = $derived(Object.keys(settings().providers));
  const models = $derived(settings().models);

  const searchProviders = [
    { name: "exa", label: "Exa", placeholder: "exa-…", hint: "$10 free credit / month" },
    { name: "serper", label: "Serper", placeholder: "…", hint: "2,500 free queries once" },
    { name: "tavily", label: "Tavily", placeholder: "tvly-…", hint: "1,000 free credits / month" },
    { name: "brave", label: "Brave", placeholder: "…", hint: "$5 free credit / month" },
  ];

  let modelInput = $state<HTMLInputElement | null>(null);
  let usage = $state<Record<string, SearchProviderUsage>>({});
  let order = $state("exa → serper → tavily → brave");

  let rotationName = $state("");
  let rotationModel = $state("");
  let rotationReasoning = $state("");

  async function addRotation(): Promise<void> {
    if (!rotationName.trim() || !rotationModel.trim()) return;
    await saveProfile({
      name: rotationName.trim(),
      model: rotationModel.trim(),
      useContext: true,
      reasoning: rotationReasoning || undefined,
    });
    rotationName = "";
    rotationModel = "";
    rotationReasoning = "";
  }

  async function refreshUsage(): Promise<void> {
    const list = await searchUsage();
    usage = Object.fromEntries(list.map((entry) => [entry.provider, entry]));
    if (list.length > 0) order = list.map((entry) => entry.provider).join(" → ");
  }

  onMount(() => {
    modelInput?.focus();
    void refreshUsage();
  });

  function dim(name: string): boolean {
    const entry = usage[name];
    return entry !== undefined && (!entry.hasKey || entry.exhausted);
  }
</script>

<div class="settings">
  <Frame title="settings" hint="⎋ close" focused>
    <header class="head">
    <h2>Settings</h2>
    <button type="button" class="ghost" onclick={onclose}>Done</button>
  </header>

  <div class="body">
    <div class="section-title">Add model</div>

    <label class="row">
      <span class="label">Provider</span>
      <select class="control" value={provider()} onchange={(event) => setProvider(event.currentTarget.value)}>
        {#each providers as name (name)}
          <option value={name}>{name}</option>
        {/each}
      </select>
    </label>

    <label class="row">
      <span class="label">Model id</span>
      <input
        bind:this={modelInput}
        class="control"
        value={modelId()}
        placeholder="deepseek/deepseek-v4.1-flash"
        oninput={(event) => setModelId(event.currentTarget.value)}
      />
    </label>

    {#if tags().length > 1}
      <label class="row">
        <span class="label">Tag</span>
        <select class="control" value={tag()} onchange={(event) => setTag(event.currentTarget.value)}>
          {#each tags() as variant (variant)}
            <option value={variant}>{variant || "none"}</option>
          {/each}
        </select>
      </label>
    {/if}

    <label class="row">
      <span class="label">Reasoning</span>
      <select
        class="control"
        value={reasoning()}
        onchange={(event) => setReasoning(event.currentTarget.value)}
      >
        {#each REASONING_LEVELS as value (value)}
          <option value={value}>{value || "default"}</option>
        {/each}
      </select>
    </label>

    <div class="row">
      <span class="label">Preview</span>
      <span class="preview">{composedModel() || "—"}</span>
    </div>

    <button type="button" class="primary" disabled={!composedModel()} onclick={() => void add().then(onclose)}>
      Add model
    </button>

    <div class="section-title">Configured models</div>
    {#if models.length === 0}
      <p class="muted">No models configured yet.</p>
    {:else}
      {#each models as entry (entry.model)}
        <ModelRow
          label={entry.model}
          reasoning={entry.reasoning ?? ""}
          onreasoning={(value) => void saveModel({ ...entry, reasoning: value || undefined })}
          onremove={() => void removeModel(entry.model)}
        />
      {/each}
    {/if}

    <div class="section-title">Rotation (Shift+Tab)</div>
    {#each settings().profiles as entry (entry.name)}
      <ModelRow
        label={entry.name}
        detail={entry.model}
        reasoning={entry.reasoning ?? ""}
        onreasoning={(value) => void saveProfile({ ...entry, reasoning: value || undefined })}
        onremove={() => void removeProfile(entry.name)}
      />
    {/each}
    <div class="add-row">
      <input class="mini" placeholder="name" bind:value={rotationName} />
      <input class="mini" placeholder="provider/model" bind:value={rotationModel} />
      <select class="mini" bind:value={rotationReasoning}>
        {#each REASONING_LEVELS as value (value)}
          <option value={value}>{value || "default"}</option>
        {/each}
      </select>
      <button
        type="button"
        class="ghost"
        disabled={!rotationName.trim() || !rotationModel.trim()}
        onclick={() => void addRotation()}
      >
        Add
      </button>
    </div>

    <div class="section-title">Web search</div>
    <p class="order">{order}</p>
    {#each searchProviders as entry (entry.name)}
      <ApiKeyRow
        provider={entry.name}
        label={entry.label}
        placeholder={entry.placeholder}
        hint={entry.hint}
        dim={dim(entry.name)}
        onchange={() => void refreshUsage()}
      />
    {/each}
    <p class="muted">Providers are tried in order; a provider without a key is skipped.</p>
    </div>
  </Frame>
</div>

<style>
  .settings {
    position: absolute;
    inset: 0;
    z-index: 11;
    display: flex;
    flex-direction: column;
    background: transparent;
    animation: rise-in var(--dur-fast) var(--ease);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 52px;
    padding: 0 var(--sp-4);
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  .body {
    flex: 1;
    padding: var(--sp-2);
    overflow-y: auto;
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
    border-radius: var(--radius-md);
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
    text-align: right;
  }

  .control:focus {
    border-color: var(--accent);
  }

  .preview {
    overflow: hidden;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .add-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-height: 40px;
    padding: 0 var(--sp-2);
  }

  .mini {
    flex: 1;
    min-width: 0;
    height: 28px;
    padding: 0 var(--sp-2);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    outline: none;
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
  }

  .mini:focus {
    border-color: var(--accent);
  }

  .primary {
    margin: var(--sp-2);
    padding: 7px var(--sp-3);
    border: none;
    border-radius: var(--radius-md);
    background: var(--accent);
    color: var(--accent-contrast);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }

  .primary:disabled {
    opacity: 0.4;
  }

  .ghost {
    padding: 4px var(--sp-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  .muted {
    margin: var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .order {
    margin: var(--sp-1) var(--sp-2);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }
</style>
