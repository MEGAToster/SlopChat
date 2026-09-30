<script lang="ts">
  import { onMount } from "svelte";
  import {
    hidePanel,
    isSticky,
    onPanelShown,
    onStickyChanged,
    toggleSticky,
  } from "./lib/ipc";

  let composer = $state<HTMLTextAreaElement | null>(null);
  let sticky = $state(false);

  function focusComposer() {
    composer?.focus();
  }

  onMount(() => {
    focusComposer();
    void isSticky().then((value) => (sticky = value));
    const onShown = onPanelShown(focusComposer);
    const onSticky = onStickyChanged((value) => (sticky = value));
    return () => {
      void onShown.then((off) => off());
      void onSticky.then((off) => off());
    };
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      void hidePanel();
      return;
    }
    if (event.ctrlKey && event.shiftKey && event.key.toLowerCase() === "p") {
      event.preventDefault();
      void toggleSticky().then((value) => (sticky = value));
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main class="panel" class:pinned={sticky}>
  <textarea
    bind:this={composer}
    class="composer"
    placeholder="Ask anything"
    rows="1"
    spellcheck="false"
  ></textarea>
  <p class="hint">
    {#if sticky}<span class="pin">pinned</span>{/if}
    <span>Esc hides · Alt+Space toggles · Ctrl+Shift+P pins</span>
  </p>
</main>
