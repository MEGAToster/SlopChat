<script lang="ts">
  import { onMount, tick } from "svelte";
  import Composer from "./components/Composer.svelte";
  import MessageList from "./components/MessageList.svelte";
  import Onboarding from "./components/Onboarding.svelte";
  import ProfileBadge from "./components/ProfileBadge.svelte";
  import StatusLine from "./components/StatusLine.svelte";
  import Toast from "./components/Toast.svelte";
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
  import { chatError, isStreaming, reportError, stop, submit } from "./lib/state/chat.svelte";
  import { configError, initSettings, setConfigError, settings } from "./lib/state/settings.svelte";
  import { activeProfile, cycleProfile } from "./lib/state/ui.svelte";

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

  async function load() {
    try {
      await initSettings();
      needsKey = !(await hasApiKey("openrouter"));
    } catch (error) {
      setConfigError(String(error));
    } finally {
      ready = true;
    }
  }

  onMount(() => {
    focusComposer();
    void isSticky().then((value) => (sticky = value));
    void load();

    let frame = 0;
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        if (content) void resizePanel(Math.ceil(content.getBoundingClientRect().height));
      });
    });
    if (content) observer.observe(content);

    const subscriptions = [
      onPanelShown(focusComposer),
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
    const value = text;
    text = "";
    await submit(value, {
      model: profile.model,
      system: profile.system,
      temperature: profile.temperature,
    });
  }

  function perform(action: Action) {
    switch (action) {
      case "hide":
        void hidePanel();
        break;
      case "stop":
        void stop();
        break;
      case "sticky":
        void toggleSticky().then((value) => (sticky = value));
        break;
      case "cycleProfile":
        cycleProfile();
        break;
      case "send":
        void handleSend();
        break;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (!ready || needsKey) {
      if (event.key === "Escape") {
        event.preventDefault();
        void hidePanel();
      }
      return;
    }

    const action = resolve("composer", event, settings().keybinds, {
      composerEmpty: text.trim().length === 0,
      chatHasMessages: false,
    });
    if (!action) return;

    event.preventDefault();
    perform(action);
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main class="panel" class:pinned={sticky}>
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
      <div class="footer">
        <StatusLine />
        <div class="footer-row">
          <ProfileBadge />
          {#if sticky}<span class="pin">pinned</span>{/if}
          {#if isStreaming()}
            <button type="button" class="stop" onclick={() => void stop()}>Stop</button>
          {/if}
        </div>
      </div>
      <Composer bind:value={text} bind:element={composer} />
    {/if}

    {#if persistentError}
      <Toast message={persistentError} />
    {:else if notice}
      <Toast message={notice} />
    {/if}
  </div>
</main>
