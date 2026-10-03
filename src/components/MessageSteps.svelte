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

  function target(step: ToolStep): string {
    const value = step.args?.url ?? step.args?.query;
    return typeof value === "string" ? value : step.name;
  }
</script>

<div class="steps">
  {#each steps as step (step.id)}
    <div class="step" class:failed={step.status === "error"}>
      <span class="glyph">{isFetch(step) ? "↓" : "⌕"}</span>
      {#if step.status === "skipped"}
        {isFetch(step) ? "fetch" : "search"} {step.summary}
      {:else if isFetch(step)}
        {#if step.status === "running"}
          fetching {domain(target(step))}…
        {:else if step.status === "error"}
          fetch failed · {step.summary}
        {:else}
          fetched {domain(target(step))} · {step.summary}
        {/if}
      {:else if step.status === "running"}
        searching “{query(step)}”…
      {:else if step.status === "error"}
        search failed · {step.summary}
      {:else}
        searched “{query(step)}” · {step.summary}
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
