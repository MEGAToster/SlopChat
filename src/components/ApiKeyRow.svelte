<script lang="ts">
  import { onMount } from "svelte";

  import { hasApiKey, storeApiKey } from "../lib/ipc";

  let {
    provider,
    label,
    placeholder,
    hint,
    dim = false,
  }: {
    provider: string;
    label: string;
    placeholder: string;
    hint: string;
    dim?: boolean;
  } = $props();

  let key = $state("");
  let stored = $state(false);

  onMount(() => {
    void hasApiKey(provider).then((value) => (stored = value));
  });

  async function save(): Promise<void> {
    if (!key.trim()) return;
    await storeApiKey(provider, key.trim());
    stored = true;
    key = "";
  }
</script>

<label class="row" class:dim>
  <span class="label">{label}</span>
  <input
    class="control"
    type="password"
    placeholder={stored ? "stored" : placeholder}
    value={key}
    oninput={(event) => (key = event.currentTarget.value)}
    onkeydown={(event) => event.key === "Enter" && void save()}
  />
</label>
<p class="muted">{hint}</p>

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
    opacity: 0.5;
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

  .muted {
    margin: 0 var(--sp-2) var(--sp-1);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }
</style>
