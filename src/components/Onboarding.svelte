<script lang="ts">
  import { onMount } from "svelte";

  import { storeApiKey } from "../lib/ipc";

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
  <h2>Add your OpenRouter API key</h2>
  <p class="muted">Stored in your Keychain, never written to disk.</p>
  <input
    bind:this={input}
    type="password"
    bind:value={key}
    placeholder="sk-or-..."
    autocomplete="off"
    onkeydown={(event) => event.key === "Enter" && save()}
  />
  <div class="onboarding-actions">
    <button type="button" onclick={save} disabled={saving || !key.trim()}>
      {saving ? "Saving…" : "Save key"}
    </button>
    <button type="button" class="ghost" onclick={onSkip}>Use Ollama instead</button>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
</div>
