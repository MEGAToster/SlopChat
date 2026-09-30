<script lang="ts">
  import { onMount } from "svelte";

  import { storeApiKey } from "../lib/ipc";
  import Kbd from "./Kbd.svelte";

  let { onSaved, onSkip }: { onSaved: () => void; onSkip: () => void } = $props();

  let key = $state("");
  let input = $state<HTMLInputElement | null>(null);
  let error = $state<string | null>(null);
  let saving = $state(false);

  onMount(() => input?.focus());

  async function save() {
    if (!key.trim() || saving) return;
    saving = true;
    error = null;
    try {
      await storeApiKey("openrouter", key.trim());
      onSaved();
    } catch (cause) {
      error = String(cause);
    } finally {
      saving = false;
    }
  }
</script>

<div class="onboarding">
  <h2>Connect OpenRouter</h2>
  <p class="sub">Stored in your Keychain, never written to disk.</p>
  <input
    bind:this={input}
    type="password"
    bind:value={key}
    placeholder="sk-or-..."
    autocomplete="off"
    onkeydown={(event) => event.key === "Enter" && save()}
  />
  {#if error}<p class="error">{error}</p>{/if}
  <div class="actions">
    <button type="button" class="primary" onclick={save} disabled={saving || !key.trim()}>
      {saving ? "Saving…" : "Save key"}
    </button>
    <button type="button" class="ghost" onclick={onSkip}>Use Ollama instead</button>
    <span class="hint"><Kbd binding="Enter" /> Save</span>
  </div>
</div>

<style>
  .onboarding {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  .sub {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  input {
    height: 36px;
    padding: 0 var(--sp-2);
    border: none;
    border-radius: var(--radius-md);
    outline: none;
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-md);
  }

  input::placeholder {
    color: var(--text-tertiary);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .primary {
    padding: 6px var(--sp-3);
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
    padding: 6px var(--sp-2);
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    cursor: pointer;
  }

  .ghost:hover {
    background: var(--surface-hover);
  }

  .hint {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: var(--fs-sm);
  }
</style>
