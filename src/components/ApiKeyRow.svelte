<script lang="ts">
  import { onMount } from "svelte";

  import { deleteApiKey, hasApiKey, storeApiKey } from "../lib/ipc";
  import Icon from "./Icon.svelte";

  let {
    provider,
    label,
    placeholder,
    hint,
    dim = false,
    onchange,
  }: {
    provider: string;
    label: string;
    placeholder: string;
    hint: string;
    dim?: boolean;
    onchange?: () => void;
  } = $props();

  let key = $state("");
  let stored = $state(false);
  let error = $state<string | null>(null);
  let busy = $state(false);

  onMount(() => void refresh());

  async function refresh(): Promise<void> {
    stored = await hasApiKey(provider).catch(() => false);
  }

  async function save(): Promise<void> {
    if (!key.trim() || busy) return;
    busy = true;
    error = null;
    try {
      await storeApiKey(provider, key.trim());
      key = "";
      await refresh();
      onchange?.();
    } catch (cause) {
      error = String(cause);
    }
    busy = false;
  }

  async function remove(): Promise<void> {
    busy = true;
    error = null;
    try {
      await deleteApiKey(provider);
      await refresh();
      onchange?.();
    } catch (cause) {
      error = String(cause);
    }
    busy = false;
  }
</script>

<div class="row" class:dim>
  <span class="label">{label}</span>
  <span class="field">
    <input
      class="control"
      type="password"
      placeholder={stored ? "•••••• saved" : placeholder}
      bind:value={key}
      oninput={() => (error = null)}
      onblur={() => void save()}
      onkeydown={(event) => event.key === "Enter" && event.currentTarget.blur()}
    />
    {#if stored}
      <button type="button" class="remove" title="Remove key" onclick={() => void remove()}>
        <Icon name="trash" size={14} />
      </button>
    {/if}
  </span>
</div>
<p class="muted">
  {#if error}
    <span class="error">{error}</span>
  {:else if stored}
    <span class="ok">key saved</span> · {hint}
  {:else}
    {hint}
  {/if}
</p>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    min-height: 40px;
    padding: 0 var(--sp-2);
    border-radius: var(--radius-md);
  }

  .row.dim {
    opacity: 0.55;
  }

  .label {
    color: var(--text-secondary);
    font-size: var(--fs-md);
  }

  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 60%;
  }

  .control {
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
    font-size: var(--fs-md);
    text-align: right;
  }

  .control:focus {
    border-color: var(--accent);
  }

  .remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .remove:hover {
    background: var(--surface-hover);
    color: var(--danger);
  }

  .muted {
    margin: 0 var(--sp-2) var(--sp-1);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .ok {
    color: var(--success);
  }

  .error {
    color: var(--danger);
  }
</style>
