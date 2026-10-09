<script lang="ts">
  import { toggleSticky } from "../lib/ipc";
  import { isAntigravity, webMode } from "../lib/status";
  import { cycleWeb, webOverride } from "../lib/state/chatStore.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { activeConfig, notifyPaidSearch } from "../lib/state/ui.svelte";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let { sticky = false, managing = false }: { sticky?: boolean; managing?: boolean } = $props();

  const config = $derived(activeConfig());
  const keybinds = $derived(settings().keybinds);
  const antigravity = $derived(isAntigravity(config?.model));
  const noTools = $derived(config?.toolCalling === false || antigravity);
  const webScoped = $derived(webOverride() !== null);

  const web = $derived(webMode(config));

  const webLabel = $derived(web === "off" ? "off" : web === "deep" ? "deep" : "web");

  const webTitle = $derived(
    antigravity
      ? "Antigravity uses its own tools"
      : config?.toolCalling === false
        ? "Tool calling is off for this profile (toolCalling: false)"
        : web === "off"
          ? "Web search off for this chat"
          : web === "deep"
            ? "Deep research on for this chat"
            : "Web search available (the model decides)",
  );

  function toggleWeb(): void {
    cycleWeb(web);
    // auto → deep turns deep research on; warn if it would cost money.
    if (web === "auto") notifyPaidSearch();
  }
</script>

<div class="controls" class:managing>
  <button
    type="button"
    class="web"
    class:scoped={webScoped}
    data-state={web}
    title={`${webTitle}${webScoped ? " (this chat)" : ""}`}
    aria-label={webTitle}
    disabled={noTools}
    onclick={toggleWeb}
  >
    <Icon name={web === "off" ? "globe-off" : "globe"} size={13} />
    <span class="label">{webLabel}</span>
    <Kbd binding={keybinds.webToggle?.[0]} />
  </button>

  <button
    type="button"
    class="sticky"
    class:on={sticky}
    title="Sticky"
    aria-label="Sticky"
    onclick={() => void toggleSticky()}
  >
    <Icon name="lock" size={13} />
    {#if sticky}<span class="label">sticky</span>{/if}
    <Kbd binding={keybinds.sticky?.[0]} />
  </button>
</div>

<style>
  .controls {
    display: inline-flex;
    align-items: center;
    flex: none;
    gap: var(--sp-2);
    margin-left: auto;
  }

  .sticky,
  .web {
    display: inline-flex;
    align-items: center;
    flex: none;
    gap: 4px;
    border: none;
    background: transparent;
    font: inherit;
    font-size: var(--fs-sm);
    cursor: default;
  }

  .label {
    font-size: var(--fs-sm);
  }

  .web[data-state="off"] {
    color: var(--text-tertiary);
  }

  .web[data-state="auto"] {
    color: var(--text-secondary);
  }

  .web[data-state="deep"] {
    color: var(--accent);
  }

  .web:disabled {
    opacity: 0.35;
  }

  .sticky {
    color: var(--text-tertiary);
    opacity: 0.55;
  }

  .sticky.on {
    color: var(--accent);
    opacity: 1;
  }

  .controls.managing .web {
    display: none;
  }
</style>
