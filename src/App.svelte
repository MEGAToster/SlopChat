<script lang="ts">
  import { onMount, tick } from "svelte";
  import Composer from "./components/Composer.svelte";
  import MessageList from "./components/MessageList.svelte";
  import Onboarding from "./components/Onboarding.svelte";
  import ProfileBadge from "./components/ProfileBadge.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import StatusLine from "./components/StatusLine.svelte";
  import Toast from "./components/Toast.svelte";
  import { messagesViewport } from "./lib/dom";
  import {
    hasApiKey,
    hidePanel,
    isSticky,
    onPanelShown,
    onStickyChanged,
    resizePanel,
    toggleSticky,
  } from "./lib/ipc";
  import { resolve, type Action } from "./lib/keymap/dispatch";
  import {
    chatError,
    init as initChat,
    isStreaming,
    lastUserMessage,
    messages,
    nextChat,
    pinChat,
    prevChat,
    regenerate,
    removeChat,
    reportError,
    setCurrentProfile,
    stop,
    submit,
  } from "./lib/state/chat.svelte";
  import { configError, initSettings, setConfigError, settings } from "./lib/state/settings.svelte";
  import {
    activeProfile,
    beginEdit,
    clampSelection,
    cycleProfile,
    editingId,
    endEdit,
    filtering,
    isExpanded,
    keyContext,
    moveSelection,
    openChatWithProfile,
    pendingDelete,
    selectActive,
    selectedChatId,
    setContext,
    setFiltering,
    setMode,
    setPendingDelete,
    syncProfileFromActive,
  } from "./lib/state/ui.svelte";

  let composer = $state<HTMLTextAreaElement | null>(null);
  let content = $state<HTMLElement | null>(null);
  let text = $state("");
  let sticky = $state(false);
  let needsKey = $state(false);
  let ready = $state(false);

  const persistentError = $derived(configError());
  let notice = $state<string | null>(null);
  let lastError: string | null = null;

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

  function focusComposer() {
    composer?.focus();
  }

  function measure() {
    if (content && !isExpanded()) {
      void resizePanel(Math.ceil(content.getBoundingClientRect().height));
    }
  }

  async function enterExpanded() {
    if (!isExpanded()) await setMode("expanded");
  }

  async function load() {
    try {
      await initSettings();
      setCurrentProfile(activeProfile()?.name ?? "default");
      needsKey = !(await hasApiKey("openrouter"));
      await initChat();
    } catch (error) {
      setConfigError(String(error));
    } finally {
      ready = true;
    }

    await tick();
    if (!needsKey) focusComposer();
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

  async function handleSend() {
    const profile = activeProfile();
    if (!profile) {
      reportError("No profile configured in settings.json");
      return;
    }
    const options = {
      profileName: profile.name,
      model: profile.model,
      system: profile.system,
      temperature: profile.temperature,
    };
    const editing = editingId();
    const value = text;
    text = "";
    if (editing) {
      endEdit();
      await regenerate(editing, value, options);
    } else {
      await submit(value, options);
    }
  }

  function startEdit() {
    const last = lastUserMessage();
    if (!last?.id) return;
    text = last.content;
    beginEdit(last.id);
    focusComposer();
  }

  function cancelEdit() {
    endEdit();
    text = "";
    focusComposer();
  }

  async function toggleSidebar() {
    if (isExpanded()) {
      setFiltering(false);
      setContext("composer");
      await setMode("compact");
      await tick();
      measure();
      focusComposer();
    } else {
      (document.activeElement as HTMLElement | null)?.blur();
      await setMode("expanded");
      selectActive();
      setFiltering(false);
      setContext("sidebar");
    }
  }

  function scrollMessages(direction: number) {
    const viewport = messagesViewport();
    if (viewport) viewport.scrollTop += direction * viewport.clientHeight * 0.5;
  }

  async function openSelected() {
    const id = selectedChatId();
    if (id) await openChatWithProfile(id);
    setFiltering(false);
    setContext("composer");
    await tick();
    focusComposer();
  }

  function perform(action: Action) {
    switch (action) {
      case "hide":
        void hidePanel();
        break;
      case "stop":
        void stop();
        break;
      case "scrollUp":
        scrollMessages(-1);
        break;
      case "scrollDown":
        scrollMessages(1);
        break;
      case "sticky":
        void toggleSticky().then((value) => (sticky = value));
        break;
      case "send":
        void handleSend();
        break;
      case "cycleProfile":
        cycleProfile();
        break;
      case "toggleSidebar":
        void toggleSidebar();
        break;
      case "editLast":
        startEdit();
        break;
      case "prevChat":
        void prevChat().then(syncProfileFromActive);
        break;
      case "nextChat":
        void nextChat().then(syncProfileFromActive);
        break;
      case "cancelEdit":
        cancelEdit();
        break;
      case "sidebarDown":
        moveSelection(1);
        break;
      case "sidebarUp":
        moveSelection(-1);
        break;
      case "openChat":
        void openSelected();
        break;
      case "pinChat": {
        const id = selectedChatId();
        if (id) void pinChat(id);
        break;
      }
      case "deleteChat": {
        const id = selectedChatId();
        if (id) setPendingDelete(id);
        break;
      }
      case "filterChats":
        void enterExpanded().then(() => {
          setContext("sidebar");
          setFiltering(true);
        });
        break;
    }
  }

  function composerIsEmpty(event: KeyboardEvent): boolean {
    const target = event.target;
    if (target instanceof HTMLTextAreaElement && target.classList.contains("composer")) {
      return target.value.trim().length === 0;
    }
    return text.trim().length === 0;
  }

  function onKeydown(event: KeyboardEvent) {
    const deleting = pendingDelete();
    if (deleting) {
      if (event.key === "y" || event.key === "Enter") {
        event.preventDefault();
        setPendingDelete(null);
        void removeChat(deleting).then(clampSelection);
      } else if (event.key === "n" || event.key === "Escape") {
        event.preventDefault();
        setPendingDelete(null);
      }
      return;
    }

    if (!ready || needsKey) {
      if (event.key === "Escape") {
        event.preventDefault();
        void hidePanel();
      }
      return;
    }

    if (filtering() && keyContext() === "sidebar") {
      if (event.key === "Escape") {
        event.preventDefault();
        setFiltering(false);
        setContext("composer");
        focusComposer();
      } else if (event.key === "Enter") {
        event.preventDefault();
        void openSelected();
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        moveSelection(1);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        moveSelection(-1);
      }
      return;
    }

    if (event.key === "Tab") event.preventDefault();

    const action = resolve(keyContext(), event, settings().keybinds, {
      composerEmpty: composerIsEmpty(event),
      chatHasMessages: messages().length > 0,
      streaming: isStreaming(),
      editing: editingId() !== null,
    });
    if (!action) return;
    event.preventDefault();
    perform(action);
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main class="panel" class:expanded={isExpanded()}>
  <div class="layout">
    {#if ready && !needsKey && isExpanded()}
      <Sidebar />
    {/if}

    <div class="content" bind:this={content}>
      {#if !ready}
        <p class="muted">Loading…</p>
      {:else if needsKey}
        <Onboarding
          onSaved={async () => {
            needsKey = false;
            await tick();
            focusComposer();
          }}
        />
      {:else}
        <MessageList />
        <div class="meta">
          <StatusLine />
          <div class="actions">
            <ProfileBadge />
            {#if sticky}<span class="pin">pinned</span>{/if}
            {#if isStreaming()}
              <button type="button" class="stop" onclick={() => void stop()}>Stop</button>
            {/if}
          </div>
        </div>
        <div class="composer-box" class:editing={editingId() !== null}>
          <Composer
            bind:value={text}
            bind:element={composer}
            disabled={isStreaming()}
            onfocus={() => setContext("composer")}
          />
        </div>
      {/if}

      {#if persistentError}
        <Toast message={persistentError} />
      {:else if notice}
        <Toast message={notice} />
      {/if}
    </div>
  </div>
</main>
