<script lang="ts">
  import { onMount } from "svelte";
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

  const providers = $derived(Object.keys(settings().providers));
  const models = $derived(settings().models);

  let { onclose }: { onclose: () => void } = $props();

  let modelInput = $state<HTMLInputElement | null>(null);

  onMount(() => modelInput?.focus());
</script>

<div class="settings">
  <div class="settings-head">
    <h2>Settings</h2>
    <button type="button" class="ghost" onclick={onclose}>Done</button>
  </div>

  <div class="settings-body">
    <section class="settings-section">
      <h3>Add model</h3>

      <label class="field">
        <span>Provider</span>
        <select value={provider()} onchange={(event) => setProvider(event.currentTarget.value)}>
          {#each providers as name (name)}
            <option value={name}>{name}</option>
          {/each}
        </select>
      </label>

      <label class="field">
        <span>Model id</span>
        <input
          bind:this={modelInput}
          value={modelId()}
          placeholder="deepseek/deepseek-v4.1-flash"
          oninput={(event) => setModelId(event.currentTarget.value)}
        />
      </label>

      {#if tags().length > 1}
        <label class="field">
          <span>Tag</span>
          <select value={tag()} onchange={(event) => setTag(event.currentTarget.value)}>
            {#each tags() as variant (variant)}
              <option value={variant}>{variant || "none"}</option>
            {/each}
          </select>
        </label>
      {/if}

      <p class="preview">{composedModel() || "—"}</p>
      <button
        type="button"
        class="primary"
        disabled={!composedModel()}
        onclick={() => void add().then(onclose)}
      >
        Add model
      </button>
    </section>

    <section class="settings-section">
      <h3>Models</h3>
      {#if models.length === 0}
        <p class="muted">No models configured yet.</p>
      {:else}
        {#each models as model (model)}
          <div class="model-row">
            <span class="model-id">{model}</span>
            <button type="button" class="ghost" onclick={() => void remove(model)}>Remove</button>
          </div>
        {/each}
      {/if}
    </section>
  </div>
</div>
