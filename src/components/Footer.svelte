<script lang="ts">
  import { formatBinding } from "../lib/keymap/format";
  import { splitModelRef, shortModel } from "../lib/modelRef";
  import { profileColor } from "../lib/profileColor";
  import { isStreamingHere, lastUsage } from "../lib/state/chatStore.svelte";
  import {
    cycleReasoningFor,
    loadReasoning,
    reasoningOverride,
  } from "../lib/state/reasoning.svelte";
  import { reasoningEffort } from "../lib/status";
  import { settings } from "../lib/state/settings.svelte";
  import {
    activeConfig,
    activeProfile,
    activeProfileIndex,
    editingId,
    keyContext,
    overrideActive,
  } from "../lib/state/ui.svelte";
  import FooterControls from "./FooterControls.svelte";
  import Kbd from "./Kbd.svelte";
  import UsageInfo from "./UsageInfo.svelte";

  let { sticky = false }: { sticky?: boolean } = $props();

  const profile = $derived(activeProfile());
  const config = $derived(activeConfig());
  const color = $derived(profileColor(profile, activeProfileIndex()));
  const fullModel = $derived(config?.model ?? "");
  const ref = $derived(splitModelRef(fullModel));
  const antigravity = $derived(ref.provider === "antigravity");
  const model = $derived(shortModel(fullModel));
  const overriding = $derived(overrideActive());
  const usage = $derived(lastUsage());
  const keybinds = $derived(settings().keybinds);
  const sidebar = $derived(keyContext() === "sidebar");
  const editing = $derived(editingId() !== null);

  const reasoning = $derived(reasoningEffort(config));
  const reasoningScoped = $derived(reasoningOverride() !== null);
  const reasoningKeys = $derived(formatBinding(keybinds.reasoning?.[0]));
  const reasoningTitle = $derived(
    `Reasoning effort: ${reasoning}${reasoningKeys ? ` (${reasoningKeys} to cycle)` : ""}${
      reasoningScoped ? " (this chat)" : ""
    }`,
  );

  $effect(() => {
    if (config?.model) void loadReasoning(config.model);
  });

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
      {#if model}
        <span class="model" class:scoped={overriding} title={overriding ? `${model} (this chat)` : model}>
          {model}
        </span>
      {/if}
      {#if model && !antigravity}
        <button
          type="button"
          class="effort"
          class:scoped={reasoningScoped}
          title={reasoningTitle}
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

    <FooterControls {sticky} managing={sidebar} />
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
    height: var(--footer-height);
    font-size: var(--fs-sm);
  }

  .mode {
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
    min-width: 0;
    overflow: hidden;
    color: var(--text-secondary);
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
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
