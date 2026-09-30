<script lang="ts">
  import { storeApiKey } from "../lib/ipc";

  let { onSaved }: { onSaved: () => void } = $props();

  let key = $state("");
  let error = $state<string | null>(null);
  let saving = $state(false);

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
    type="password"
    bind:value={key}
    placeholder="sk-or-..."
    autocomplete="off"
    onkeydown={(event) => event.key === "Enter" && save()}
  />
  <button type="button" onclick={save} disabled={saving || !key.trim()}>
    {saving ? "Saving…" : "Save key"}
  </button>
  {#if error}<p class="error">{error}</p>{/if}
</div>
