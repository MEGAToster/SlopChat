<script lang="ts">
  import { formatBinding } from "../lib/keymap/format";

  let { binding }: { binding?: string } = $props();
  const text = $derived(formatBinding(binding));
</script>

{#if text}<span class="kbd">{text}</span>{/if}

<style>
  .kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
    color: var(--text-secondary);
    font-size: var(--fs-xs);
    line-height: 1;
  }
</style>
