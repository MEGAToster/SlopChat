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
  import {
    hasApiKey,
    isSticky,
    onModelFallback,
    onPanelShown,
    onSearchPaid,
    onSearchUnavailable,
    onStickyChanged,
    setPanelExpanded,
  } from "./lib/ipc";
  import { createKeyHandler } from "./lib/keymap/handler";
  import {
    activateSettingsFocus,
    classifyElement,
    moveSettingsFocus,
    setFocusedSummary,
  } from "./lib/settingsFocus";
  import { createActions } from "./lib/state/actions.svelte";
  import { init as initChat, resumeOnShow } from "./lib/state/chatHistory.svelte";
  import { chatError, isStreaming, messages } from "./lib/state/chatStore.svelte";
  import {
    back as paletteBack,
    isOpen as paletteIsOpen,
    mode as paletteMode,
    move as paletteMove,
    select as paletteSelect,
  } from "./lib/state/palette.svelte";
  import { configError, initSettings, setConfigError, settings } from "./lib/state/settings.svelte";
  import { measurePanel } from "./lib/state/overlays.svelte";
  import {
    backFromEditor,
    isOpen as settingsPanelIsOpen,
  } from "./lib/state/settingsPanel.svelte";
  import { applyThemeFor, loadUser } from "./lib/state/theme.svelte";
  import {
    clearNotice,
    editingId,
    filtering,
    isExpanded,
    keyContext,
    notify,
    pendingDelete,
    selectActive,
    setContext,
    uiNotice,
    uiNoticeId,
    uiNoticeKind,
  } from "./lib/state/ui.svelte";
  import type { KeymapState } from "./lib/keymap/dispatch";

  let content = $state<HTMLElement | null>(null);
  let text = $state("");
  let sticky = $state(false);
  let needsKey = $state(false);
  let ready = $state(false);

  const actions = createActions({ get: () => text, set: (value) => (text = value) });

  const persistentError = $derived(configError());
  const uiNoticeText = $derived(uiNotice());
  let wasStreaming = false;

  $effect(() => {
    const error = chatError();
    if (error) notify(error, "danger");
  });

  $effect(() => {
    // Read the id so an identical repeat also restarts the timer.
    if (uiNoticeId() === 0 || !uiNotice()) return;
    const timer = setTimeout(clearNotice, 5000);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    applyThemeFor(settings().theme);
  });

  // Re-apply window sizing (width/height/ratio) when the config changes.
  $effect(() => {
    void settings().window.compactWidth;
    void settings().window.compactHeight;
    void settings().window.expandedScreenRatio;
    requestAnimationFrame(() => {
      if (isExpanded()) void setPanelExpanded(true);
      else measurePanel();
    });
  });

  // Compact panel follows the thread height after each change.
  $effect(() => {
    void messages().length;
    requestAnimationFrame(measurePanel);
  });

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

  // Retried across a frame because the panel may not be key when
  // `panel://shown` arrives, which makes a single focus() a no-op.
  function focusComposerSoon(): void {
    void tick().then(() => {
      requestAnimationFrame(() => {
        focusComposer();
        setTimeout(focusComposer, 0);
      });
    });
  }

  onMount(() => {
    focusComposer();
    void isSticky().then((value) => (sticky = value));
    void load();

    let frame = 0;
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(measurePanel);
    });
    if (content) observer.observe(content);

    const subscriptions = [
      onPanelShown(() => {
        const overlay =
          paletteIsOpen() ||
          settingsPanelIsOpen() ||
          pendingDelete() !== null ||
          filtering();
        if (!overlay) setContext("composer");
        void resumeOnShow({ hasDraft: text.trim().length > 0, streaming: isStreaming() }).then(
          () => {
            if (isExpanded()) selectActive();
            if (!overlay) focusComposerSoon();
          },
        );
      }),
      onStickyChanged((value) => (sticky = value)),
      onSearchUnavailable((message) => notify(message, "warning")),
      onSearchPaid((message) => notify(message, "warning")),
      onModelFallback((message) => notify(message, "warning")),
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
    settingsBack: backFromEditor,
    settingsFocus: () => classifyElement(document.activeElement),
    blocked: () => !ready || needsKey,
    filterActive: filtering,
    confirmActive: () => pendingDelete() !== null,
    actions: {
      perform: actions.perform,
      paletteMove,
      paletteActivate: actions.paletteActivateAndClose,
      paletteBack,
      paletteClose: actions.closePalette,
      closeSettings: actions.closeSettings,
      closeFilter: actions.closeFilter,
      openSelected: actions.openSelected,
      confirmDelete: actions.confirmDelete,
      cancelDelete: actions.cancelDelete,
      escChain: actions.escChain,
      settingsMove: moveSettingsFocus,
      settingsActivate: activateSettingsFocus,
      settingsExpand: () => setFocusedSummary(true),
      settingsCollapse: () => setFocusedSummary(false),
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

      {#if uiNoticeText}
        <Toast message={uiNoticeText} kind={uiNoticeKind()} />
      {/if}
      {#if persistentError}
        <Toast message={persistentError} kind="warning" pinned />
      {/if}
    </div>
  </div>

  {#if paletteIsOpen()}
    <Palette
      onactivate={(selected) => {
        paletteSelect(selected);
        actions.paletteActivateAndClose();
      }}
    />
  {/if}

  {#if settingsPanelIsOpen()}
    <Settings onclose={actions.closeSettings} />
  {/if}
</main>
