<script lang="ts">
  import { summarizeSteps } from "../lib/steps";
  import { domain } from "../lib/text";
  import type { ToolStep } from "../lib/types";

  let { steps, done = false }: { steps: ToolStep[]; done?: boolean } = $props();

  let open = $state(false);
  const summary = $derived(summarizeSteps(steps));

  function query(step: ToolStep): string {
    const value = step.args?.query;
    return typeof value === "string" ? value : step.name;
  }

  function isFetch(step: ToolStep): boolean {
    return step.name === "web_fetch";
  }

  function isSearch(step: ToolStep): boolean {
    return step.name === "web_search";
  }

  function target(step: ToolStep): string {
    const value = step.args?.url ?? step.args?.query;
    return typeof value === "string" ? value : step.name;
  }

  // Antigravity's own tools name their target differently per tool.
  function genericTarget(step: ToolStep): string {
    const args = step.args ?? {};
    for (const key of ["CommandLine", "path", "url", "query"]) {
      const value = args[key];
      if (typeof value === "string" && value) return value;
    }
    return "";
  }
</script>

{#snippet row(step: ToolStep)}
  <div class="step" class:failed={step.status === "error"}>
    <span class="glyph">{isFetch(step) ? "↓" : isSearch(step) ? "⌕" : "›"}</span>
    {#if step.status === "skipped"}
      {isFetch(step) ? "fetch" : isSearch(step) ? "search" : step.name} {step.summary}
    {:else if isFetch(step)}
      {#if step.status === "running"}
        fetching {domain(target(step))}…
      {:else if step.status === "error"}
        fetch failed · {step.summary}
      {:else}
        fetched {domain(target(step))} · {step.summary}
      {/if}
    {:else if isSearch(step)}
      {#if step.status === "running"}
        searching “{query(step)}”…
      {:else if step.status === "error"}
        search failed · {step.summary}
      {:else}
        searched “{query(step)}” · {step.summary}
      {/if}
    {:else}
      {#if step.status === "running"}
        {step.name}{genericTarget(step) ? ` ${genericTarget(step)}` : ""}…
      {:else if step.status === "error"}
        {step.name} failed · {step.summary}
      {:else}
        {step.name} · {step.summary}
      {/if}
    {/if}
    {#if step.note}<span class="note">{step.note}</span>{/if}
  </div>
{/snippet}

<div class="steps">
  {#if done}
    <button type="button" class="summary" onclick={() => (open = !open)}>
      {open ? "▾" : "▸"} {summary}
    </button>
    {#if open}
      {#each steps as step (step.id)}{@render row(step)}{/each}
    {/if}
  {:else}
    {#if steps.length > 3}
      <div class="step earlier">+{steps.length - 3} earlier</div>
    {/if}
    {#each steps.slice(-3) as step (step.id)}{@render row(step)}{/each}
  {/if}
</div>

<style>
  .steps {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    margin-bottom: var(--sp-1);
  }

  .summary {
    align-self: flex-start;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    font: inherit;
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  .summary:hover {
    color: var(--text-secondary);
  }

  .step {
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
    overflow-wrap: anywhere;
  }

  .earlier {
    font-style: italic;
  }

  .step.failed {
    color: var(--danger);
  }

  .note {
    display: block;
    margin-left: 16px;
    color: var(--text-tertiary);
  }

  .glyph {
    margin-right: 4px;
  }
</style>
