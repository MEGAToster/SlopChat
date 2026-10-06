<script lang="ts">
  import { domain } from "../lib/text";
  import type { ToolStep } from "../lib/types";

  let { steps }: { steps: ToolStep[] } = $props();

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

<div class="steps">
  {#each steps as step (step.id)}
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
  {/each}
</div>

<style>
  .steps {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    margin-bottom: var(--sp-1);
  }

  .step {
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
    overflow-wrap: anywhere;
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
