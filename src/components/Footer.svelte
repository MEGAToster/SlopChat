<script lang="ts">
  import { splitModelRef, shortModel } from "../lib/modelRef";
  import { profileColor } from "../lib/profileColor";
  import {
    isStreamingHere,
    lastUsage,
    cycleWeb,
    webOverride,
  } from "../lib/state/chatStore.svelte";
  import {
    cycleReasoningFor,
    loadReasoning,
    reasoningInfo,
    reasoningOverride,
  } from "../lib/state/reasoning.svelte";
  import { followOutput, view } from "../lib/state/view.svelte";
  import { effectiveWebMode } from "../lib/webSearch";
  import { settings } from "../lib/state/settings.svelte";
  import {
    activeConfig,
    activeProfile,
    activeProfileIndex,
    editingId,
    keyContext,
    overrideActive,
  } from "../lib/state/ui.svelte";
  import { toggleSticky } from "../lib/ipc";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";
  import UsageInfo from "./UsageInfo.svelte";

  let { sticky = false }: { sticky?: boolean } = $props();

  const profile = $derived(activeProfile());
  const config = $derived(activeConfig());
  const color = $derived(profileColor(profile, activeProfileIndex()));
  const fullModel = $derived(config?.model ?? "");
  const ref = $derived(splitModelRef(fullModel));
  const model = $derived(shortModel(fullModel));
  const overriding = $derived(overrideActive());
  const usage = $derived(lastUsage());
  const keybinds = $derived(settings().keybinds);
  const sidebar = $derived(keyContext() === "sidebar");
  const editing = $derived(editingId() !== null);

  const info = $derived(config ? reasoningInfo(config.model) : undefined);
  const reasoning = $derived(
    reasoningOverride() ?? config?.reasoning ?? info?.defaultEffort ?? "auto",
  );

  $effect(() => {
    if (config?.model) void loadReasoning(config.model);
  });

  const web = $derived.by(() => {
    const mode = effectiveWebMode(
      config?.toolCalling !== false,
      webOverride(),
      config?.webSearch,
    );
    return mode === "off" ? "off" : mode === "always" ? "on" : "auto";
  });

  const webTitle = $derived(
    config?.toolCalling === false
      ? "Tool calling is off for this profile (toolCalling: false)"
      : web === "off"
        ? "Web search off for this chat"
        : web === "on"
          ? "Web search forced on for this chat"
          : "Web search available (the model decides)",
  );

  const mode = $derived.by(() => {
    if (isStreamingHere()) return { label: "STREAM", tone: "orange" };
    if (editing) return { label: "EDIT", tone: "yellow" };
    if (sidebar) return { label: "NORMAL", tone: "blue" };
    return { label: "INSERT", tone: "green" };
  });

</script>

<footer class="footer">
  <div class="column inner">
    <span class="mode" data-tone={mode.tone}>{mode.label}</span>

    <div class="profile">
      {#if !overriding}
        <span class="dot" style={`background: var(--c-${color})`}></span>
        <span class="name">{profile?.name ?? "no profile"}</span>
      {/if}
      {#if model}<span class="model">{model}</span>{/if}
      {#if model}
        <button
          type="button"
          class="effort"
          title={`Reasoning effort: ${reasoning} (${keybinds.reasoning?.[0] ?? ""} to cycle)`}
          onclick={() => config && void cycleReasoningFor(config.model)}
        >
          {reasoning}
        </button>
      {/if}
    </div>

    <UsageInfo {usage} apiProvider={ref.provider} />

    <div class="hints">
      {#if sidebar}
        <span><Kbd binding="Enter" /> Open</span>
        <span><Kbd binding="p" /> Pin</span>
        <span><Kbd binding="d" /> Delete</span>
      {:else if editing}
        <span><Kbd binding="Enter" /> Resend</span>
        <span><Kbd binding="Esc" /> Cancel</span>
      {:else if isStreamingHere()}
        <span><Kbd binding={keybinds.stop?.[0]} /> Stop</span>
      {/if}
    </div>

    <div class="controls" class:managing={sidebar}>
      <button
        type="button"
        class="follow"
        class:on={view.tracking}
        title="Follow output / jump to present"
        onclick={followOutput}
      >
        <Icon name="follow" size={13} />
        <Kbd binding={keybinds.follow?.[0]} />
      </button>

      <button
        type="button"
        class="sticky"
        class:on={sticky}
        title="Sticky"
        onclick={() => void toggleSticky()}
      >
        <Icon name="lock" size={13} />
        <Kbd binding={keybinds.sticky?.[0]} />
      </button>

      <button
        type="button"
        class="web"
        data-state={web}
        title={webTitle}
        disabled={config?.toolCalling === false}
        onclick={cycleWeb}
      >
        <Icon name={web === "off" ? "globe-off" : "globe"} size={13} />
        <Kbd binding={keybinds.webToggle?.[0]} />
      </button>
    </div>
  </div>
</footer>

<style>
  .footer {
    width: 100%;
    border-top: 1px solid var(--border);
  }

  .inner {
    display: flex;
    align-items: center;
    flex-wrap: nowrap;
    gap: var(--sp-2);
    height: 36px;
    font-size: var(--fs-sm);
  }

  .mode {
    display: none;
  }

  .controls {
    display: inline-flex;
    align-items: center;
    flex: none;
    gap: var(--sp-2);
    margin-left: auto;
  }

  .follow,
  .sticky,
  .web {
    display: inline-flex;
    align-items: center;
    flex: none;
    gap: 4px;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    font: inherit;
    font-size: var(--fs-sm);
    cursor: default;
  }

  .follow,
  .sticky {
    opacity: 0.55;
  }

  .follow.on,
  .sticky.on {
    color: var(--accent);
    opacity: 1;
  }

  /* Web: the globe keeps its three stages; the keybind only lights up when on. */
  .web :global(svg),
  .web :global(.kbd) {
    opacity: 0.55;
  }

  .web[data-state="auto"] :global(svg) {
    opacity: 1;
  }

  .web[data-state="on"] {
    color: var(--accent);
  }

  .web[data-state="on"] :global(svg),
  .web[data-state="on"] :global(.kbd) {
    opacity: 1;
  }

  .web:disabled :global(svg),
  .web:disabled :global(.kbd) {
    opacity: 0.35;
  }

  .controls.managing .web {
    display: none;
  }

  .profile {
    display: inline-flex;
    align-items: center;
    flex: 0 1 auto;
    gap: 6px;
    min-width: 0;
  }

  .dot {
    flex: none;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  /* color comes from the profile palette token inline */

  .name {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .model {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    color: var(--text-tertiary);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .effort {
    padding: 0 5px;
    border: none;
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
    color: var(--text-secondary);
    font: inherit;
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    cursor: default;
  }

  .hints {
    display: inline-flex;
    align-items: center;
    flex: none;
    gap: var(--sp-1);
    color: var(--text-tertiary);
  }

  .hints span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

</style>
