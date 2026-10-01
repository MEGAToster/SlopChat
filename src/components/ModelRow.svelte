<script lang="ts">
  import { REASONING_LEVELS } from "../lib/reasoningLevels";
  import Icon from "./Icon.svelte";

  let {
    label,
    detail = "",
    reasoning = "",
    onreasoning,
    onremove,
  }: {
    label: string;
    detail?: string;
    reasoning?: string;
    onreasoning: (value: string) => void;
    onremove: () => void;
  } = $props();
</script>

<div class="row">
  <span class="names">
    <span class="label">{label}</span>
    {#if detail}<span class="detail">{detail}</span>{/if}
  </span>
  <select
    class="control"
    value={reasoning}
    title="Default reasoning effort"
    onchange={(event) => onreasoning(event.currentTarget.value)}
  >
    {#each REASONING_LEVELS as value (value)}
      <option value={value}>{value || "default"}</option>
    {/each}
  </select>
  <button type="button" class="icon-button" title="Remove" onclick={onremove}>
    <Icon name="trash" size={15} />
  </button>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-height: 40px;
    padding: 0 var(--sp-2);
    border-radius: var(--radius-md);
  }

  .names {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .label {
    overflow: hidden;
    font-size: var(--fs-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .detail {
    overflow: hidden;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .control {
    flex: none;
    width: 110px;
    height: 28px;
    padding: 0 var(--sp-2);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    outline: none;
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
  }

  .control:focus {
    border-color: var(--accent);
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .icon-button:hover {
    background: var(--surface-hover);
    color: var(--danger);
  }
</style>
