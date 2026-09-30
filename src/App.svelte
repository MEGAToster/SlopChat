<script lang="ts">
  import { onMount } from "svelte";
  import Toast from "./components/Toast.svelte";
  import { hidePanel, isSticky, onPanelShown, onStickyChanged, toggleSticky } from "./lib/ipc";
  import { resolve, type Action } from "./lib/keymap/dispatch";
  import {
    configError,
    initSettings,
    setConfigError,
    settings,
  } from "./lib/state/settings.svelte";

  let composer = $state<HTMLTextAreaElement | null>(null);
  let text = $state("");
  let sticky = $state(false);

  const activeError = $derived(configError());

  function focusComposer() {
    composer?.focus();
  }

  onMount(() => {
    focusComposer();
    void isSticky().then((value) => (sticky = value));
    void initSettings().catch((error) => setConfigError(String(error)));
    const subscriptions = [
      onPanelShown(focusComposer),
      onStickyChanged((value) => (sticky = value)),
    ];
    return () => {
      for (const subscription of subscriptions) void subscription.then((off) => off());
    };
  });

  function perform(action: Action) {
    switch (action) {
      case "hide":
        void hidePanel();
        break;
      case "sticky":
        void toggleSticky().then((value) => (sticky = value));
        break;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    const action = resolve("composer", event, settings().keybinds, {
      composerEmpty: text.trim().length === 0,
      chatHasMessages: false,
    });
    if (!action) return;
    if (action !== "send") event.preventDefault();
    perform(action);
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main class="panel" class:pinned={sticky}>
  <textarea
    bind:this={composer}
    bind:value={text}
    class="composer"
    placeholder="Ask anything"
    rows="1"
    spellcheck="false"
  ></textarea>
  <p class="hint">
    {#if sticky}<span class="pin">pinned</span>{/if}
    <span>Esc hides · Alt+Space toggles · Ctrl+Shift+P pins</span>
  </p>
  {#if activeError}<Toast message={activeError} />{/if}
</main>
