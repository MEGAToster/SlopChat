<script lang="ts">
  import Kbd from "./Kbd.svelte";

  let {
    value = $bindable(),
    element = $bindable(null),
    placeholder = "Ask anything",
    onfocus,
    editing = false,
  }: {
    value: string;
    element?: HTMLTextAreaElement | null;
    placeholder?: string;
    onfocus?: () => void;
    editing?: boolean;
  } = $props();

  const MAX_HEIGHT = 8 * 21;

  $effect(() => {
    void value;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${Math.min(element.scrollHeight, MAX_HEIGHT)}px`;
  });
</script>

<div class="composer-box" class:editing>
  {#if editing}<span class="editing-label">Editing message</span>{/if}
  <div class="row">
    <textarea
      bind:this={element}
      bind:value
      class="input"
      {placeholder}
      rows="1"
      spellcheck="false"
      onfocus={() => onfocus?.()}
    ></textarea>
    <span class="enter" class:dim={!value.trim()}><Kbd binding="Enter" /></span>
  </div>
</div>

<style>
  .composer-box {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
  }

  .composer-box.editing {
    padding-left: var(--sp-2);
    border-left: 2px solid var(--accent);
  }

  .editing-label {
    color: var(--text-tertiary);
    font-size: var(--fs-xs);
  }

  .row {
    display: flex;
    align-items: flex-end;
    gap: var(--sp-2);
  }

  .input {
    flex: 1;
    max-height: 168px;
    resize: none;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: var(--fs-lg);
    line-height: 1.4;
  }

  .input::placeholder {
    color: var(--text-tertiary);
  }

  .enter {
    padding-bottom: 2px;
    transition: opacity var(--dur-fast) var(--ease);
  }

  .enter.dim {
    opacity: 0.4;
  }
</style>
