<script lang="ts">
  import { onMount, tick } from "svelte";
  import Onboarding from "./components/Onboarding.svelte";
  import Palette from "./components/Palette.svelte";
  import Settings from "./components/Settings.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Thread from "./components/Thread.svelte";
  import Toast from "./components/Toast.svelte";
  import { handleWindowClick } from "./lib/clicks";
  import { focusComposer } from "./lib/dom";
  import { hasApiKey, isSticky, onPanelShown, onStickyChanged, resizePanel } from "./lib/ipc";
  import { createKeyHandler } from "./lib/keymap/handler";
  import { createActions } from "./lib/state/actions.svelte";
  import { chatError, init as initChat, isStreaming, messages } from "./lib/state/chat.svelte";
  import {
    back as paletteBack,
    isOpen as paletteIsOpen,
    mode as paletteMode,
    move as paletteMove,
    select as paletteSelect,
    activate as paletteActivate,
  } from "./lib/state/palette.svelte";
  import { configError, initSettings, setConfigError, settings } from "./lib/state/settings.svelte";
  import { isOpen as settingsPanelIsOpen } from "./lib/state/settingsPanel.svelte";
  import { applyThemeFor, loadUser } from "./lib/state/theme.svelte";
  import {
    editingId,
    filtering,
    isExpanded,
    keyContext,
    pendingDelete,
    setContext,
  } from "./lib/state/ui.svelte";
  import type { KeymapState } from "./lib/keymap/dispatch";

  let content = $state<HTMLElement | null>(null);
  let text = $state("");
  let sticky = $state(false);
  let needsKey = $state(false);
  let ready = $state(false);

  const actions = createActions({ get: () => text, set: (value) => (text = value) });

  const persistentError = $derived(configError());
  let notice = $state<string | null>(null);
  let lastError: string | null = null;
  let wasStreaming = false;

  $effect(() => {
    const error = chatError();
    if (error === null) {
      lastError = null;
      return;
    }
    if (error !== lastError) {
      lastError = error;
      notice = error;
    }
  });

  $effect(() => {
    if (!notice) return;
    const timer = setTimeout(() => (notice = null), 5000);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    applyThemeFor(settings().theme);
  });

  $effect(() => {
    void messages().length;
    requestAnimationFrame(measure);
  });

  function measure() {
    if (content && !isExpanded()) {
      void resizePanel(Math.ceil(content.getBoundingClientRect().height));
    }
  }

  $effect(() => {
    const streaming = isStreaming();
    if (wasStreaming && !streaming && keyContext() === "composer") {
      void tick().then(focusComposer);
    }
    wasStreaming = streaming;
  });

  async function load() {
    try {
      await initSettings();
      await loadUser();
      await initChat();
    } catch (error) {
      setConfigError(String(error));
    }

    ready = true;
    await tick();
    focusComposer();

    // Keychain lookups can be slow or prompt, so resolve the key in the
    // background instead of blocking the first paint.
    try {
      needsKey = !(await hasApiKey("openrouter"));
    } catch {
      needsKey = false;
    }
    if (!needsKey) {
      await tick();
      focusComposer();
    }
  }

  onMount(() => {
    focusComposer();
    void isSticky().then((value) => (sticky = value));
    void load();

    let frame = 0;
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(measure);
    });
    if (content) observer.observe(content);

    const subscriptions = [
      onPanelShown(() => {
        if (keyContext() === "composer") focusComposer();
      }),
      onStickyChanged((value) => (sticky = value)),
    ];

    return () => {
      observer.disconnect();
      cancelAnimationFrame(frame);
      for (const subscription of subscriptions) void subscription.then((off) => off());
    };
  });

  function keymapState(): KeymapState {
    const active = document.activeElement;
    const inComposer =
      active instanceof HTMLTextAreaElement && active.closest(".composer-box") !== null;
    const empty = inComposer ? active.value.trim().length === 0 : text.trim().length === 0;
    return {
      composerEmpty: empty,
      chatHasMessages: messages().length > 0,
      streaming: isStreaming(),
      editing: editingId() !== null,
    };
  }

  const onKeydown = createKeyHandler({
    keybinds: () => settings().keybinds,
    state: keymapState,
    context: keyContext,
    paletteOpen: paletteIsOpen,
    paletteMode,
    settingsOpen: settingsPanelIsOpen,
    blocked: () => !ready || needsKey,
    filterActive: filtering,
    confirmActive: () => pendingDelete() !== null,
    actions: {
      perform: actions.perform,
      paletteMove,
      paletteActivate: () => {
        void paletteActivate()
          .then((shouldClose) => {
            if (shouldClose) actions.closePalette();
          })
          .catch(actions.closePalette);
      },
      paletteBack,
      paletteClose: actions.closePalette,
      closeSettings: actions.closeSettings,
      closeFilter: actions.closeFilter,
      openSelected: actions.openSelected,
      confirmDelete: actions.confirmDelete,
      cancelDelete: actions.cancelDelete,
      escChain: actions.escChain,
    },
  });

</script>

<svelte:window onkeydown={onKeydown} onclick={handleWindowClick} />

<main class="panel" class:expanded={isExpanded()} class:overlay={paletteIsOpen() || settingsPanelIsOpen()}>
  <div class="layout">
    {#if ready && !needsKey && isExpanded()}
      <Sidebar />
    {/if}

    <div class="content" bind:this={content}>
      {#if !ready}
        <div class="column loading">Loading…</div>
      {:else if needsKey}
        <Onboarding
          onSaved={async () => {
            needsKey = false;
            await tick();
            focusComposer();
          }}
          onSkip={async () => {
            needsKey = false;
            await tick();
            focusComposer();
          }}
        />
      {:else}
        <Thread
          bind:text
          {sticky}
          expanded={isExpanded()}
          onfocus={() => setContext("composer")}
        />
      {/if}

      {#if persistentError}
        <Toast message={persistentError} kind="warning" />
      {:else if notice}
        <Toast message={notice} />
      {/if}
    </div>
  </div>

  {#if paletteIsOpen()}
    <Palette
      onactivate={(selected) => {
        paletteSelect(selected);
        void paletteActivate()
          .then((shouldClose) => {
            if (shouldClose) actions.closePalette();
          })
          .catch(actions.closePalette);
      }}
    />
  {/if}

  {#if settingsPanelIsOpen()}
    <Settings onclose={actions.closeSettings} />
  {/if}
</main>
