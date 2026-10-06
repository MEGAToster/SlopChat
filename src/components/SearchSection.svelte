<script lang="ts">
  import { onMount } from "svelte";

  import { searchUsage, setSearxngUrl } from "../lib/ipc";
  import { settings } from "../lib/state/settings.svelte";
  import type { SearchProviderUsage } from "../lib/types";
  import ApiKeyRow from "./ApiKeyRow.svelte";

  const searchProviders = [
    { name: "exa", label: "Exa", placeholder: "exa-…", hint: "$10 free credit / month" },
    { name: "tavily", label: "Tavily", placeholder: "tvly-…", hint: "1,000 free credits / month" },
    { name: "brave", label: "Brave", placeholder: "…", hint: "$5 free credit / month" },
    { name: "firecrawl", label: "Firecrawl", placeholder: "fc-…", hint: "500 free credits / month" },
    { name: "serper", label: "Serper", placeholder: "…", hint: "2,500 free queries once" },
  ];

  let usage = $state<Record<string, SearchProviderUsage>>({});
  let order = $state("searxng → exa → tavily → brave → firecrawl → serper");

  async function refreshUsage(): Promise<void> {
    const list = await searchUsage();
    usage = Object.fromEntries(list.map((entry) => [entry.provider, entry]));
    if (list.length > 0) order = list.map((entry) => entry.provider).join(" → ");
  }

  onMount(() => {
    void refreshUsage();
  });

  function dim(name: string): boolean {
    const entry = usage[name];
    return entry !== undefined && (!entry.hasKey || entry.exhausted);
  }
</script>

<div class="section-title">Web search</div>
<p class="order">{order}</p>

<label class="row">
  <span class="label">SearXNG URL</span>
  <input
    class="control"
    placeholder="http://localhost:8080"
    value={settings().search.searxng.baseUrl}
    onchange={(event) => void setSearxngUrl(event.currentTarget.value)}
  />
</label>

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

<style>
  .order {
    margin: var(--sp-1) var(--sp-2);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-height: 36px;
    padding: 0 var(--sp-2);
  }

  .label {
    flex: none;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  .control {
    flex: 1;
    height: 28px;
    padding: 0 var(--sp-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
  }

  .control:focus {
    outline: 1px solid var(--accent);
  }

  .muted {
    margin: var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }
</style>
