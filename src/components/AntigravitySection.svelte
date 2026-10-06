<script lang="ts">
  import { onMount } from "svelte";

  import { antigravityStatus } from "../lib/ipc";
  import type { AntigravityStatus } from "../lib/types";

  let status = $state<AntigravityStatus | null>(null);
  let loading = $state(false);
  let error = $state(false);

  async function refresh(): Promise<void> {
    loading = true;
    error = false;
    try {
      status = await antigravityStatus();
    } catch {
      status = null;
      error = true;
    }
    loading = false;
  }

  onMount(() => {
    void refresh();
  });
</script>

<div class="section-head">
  <div class="section-title">Antigravity</div>
  <button type="button" class="ghost" disabled={loading} onclick={() => void refresh()}>
    {loading ? "Checking…" : "Refresh"}
  </button>
</div>

{#if status?.path}
  <p class="line">{status.path}{status.version ? ` · v${status.version}` : ""}</p>
{:else if status}
  <p class="line">
    Not installed — run `curl -fsSL https://antigravity.google/cli/install.sh | bash`
  </p>
{:else if error}
  <p class="muted">Could not check Antigravity.</p>
{:else}
  <p class="muted">Checking…</p>
{/if}

{#if status?.note}
  <p class="muted">{status.note}</p>
{/if}

<p class="muted">Sign in: run `agy` once in a terminal.</p>

<style>
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: var(--sp-2);
  }

  .line {
    margin: var(--sp-1) var(--sp-2);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    word-break: break-all;
  }

  .muted {
    margin: var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
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
</style>
