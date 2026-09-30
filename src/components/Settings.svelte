<script lang="ts">
  import { onMount } from "svelte";
  import { hasApiKey, storeApiKey } from "../lib/ipc";
  import {
    add,
    composedModel,
    modelId,
    provider,
    remove,
    setModelId,
    setProvider,
    setTag,
    tag,
    tags,
  } from "../lib/state/settingsPanel.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import Frame from "./Frame.svelte";
  import Icon from "./Icon.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const providers = $derived(Object.keys(settings().providers));
  const models = $derived(settings().models);

  let modelInput = $state<HTMLInputElement | null>(null);
  let exaKey = $state("");
  let exaStored = $state(false);

  onMount(() => {
    modelInput?.focus();
    void hasApiKey("exa").then((stored) => (exaStored = stored));
  });

  async function saveExa() {
    if (!exaKey.trim()) return;
    await storeApiKey("exa", exaKey.trim());
    exaStored = true;
    exaKey = "";
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
      {#each models as model (model)}
        <div class="row">
          <span class="model-id">{model}</span>
          <button type="button" class="icon-button" title="Remove" onclick={() => void remove(model)}>
            <Icon name="trash" size={15} />
          </button>
        </div>
      {/each}
    {/if}

    <div class="section-title">Search</div>
    <label class="row">
      <span class="label">Exa API key</span>
      <input
        class="control"
        type="password"
        placeholder={exaStored ? "stored" : "exa-…"}
        value={exaKey}
        oninput={(event) => (exaKey = event.currentTarget.value)}
        onkeydown={(event) => event.key === "Enter" && void saveExa()}
      />
    </label>
    <p class="muted">With no card attached to your Exa account, it can't overspend.</p>
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

  .model-id {
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .icon-button:hover {
    background: var(--surface-hover);
    color: var(--danger);
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
</style>
