<script lang="ts">
  import { profileColor } from "../lib/profileColor";
  import {
    isStreamingHere,
    lastUsage,
    messages,
    cycleWeb,
    webOverride,
  } from "../lib/state/chatStore.svelte";
  import {
    cycleReasoningFor,
    loadReasoning,
    reasoningInfo,
    reasoningOverride,
  } from "../lib/state/reasoning.svelte";
  import { followOutput, tracking } from "../lib/state/view.svelte";
  import { effectiveWebMode } from "../lib/webSearch";
  import { settings } from "../lib/state/settings.svelte";
  import {
    activeConfig,
    activeProfile,
    activeProfileIndex,
    editingId,
    keyContext,
  } from "../lib/state/ui.svelte";
  import { toggleSticky } from "../lib/ipc";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let { sticky = false }: { sticky?: boolean } = $props();

  const profile = $derived(activeProfile());
  const config = $derived(activeConfig());
  const color = $derived(profileColor(profile, activeProfileIndex()));
  const fullModel = $derived(config?.model ?? "");
  const model = $derived(fullModel.split("/").pop() ?? "");
  const usage = $derived(lastUsage());
  const provider = $derived(usage?.provider ?? fullModel.split("/")[0] ?? "");
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

  const total = $derived.by(() => {
    let tokensIn = 0;
    let tokensOut = 0;
    let cost = 0;
    let hasCost = false;
    for (const message of messages()) {
      if (!message.usage) continue;
      tokensIn += message.usage.tokensIn;
      tokensOut += message.usage.tokensOut;
      if (message.usage.cost !== null) {
        cost += message.usage.cost;
        hasCost = true;
      }
    }
    return { tokensIn, tokensOut, cost, hasCost };
  });

  const totalText = $derived(
    `total ↑${total.tokensIn} ↓${total.tokensOut}${total.hasCost ? ` · ${money(total.cost)}` : ""}`,
  );

  const mode = $derived.by(() => {
    if (isStreamingHere()) return { label: "STREAM", tone: "orange" };
    if (editing) return { label: "EDIT", tone: "yellow" };
    if (keyContext() === "palette") return { label: "CMD", tone: "purple" };
    if (sidebar) return { label: "NORMAL", tone: "blue" };
    return { label: "INSERT", tone: "green" };
  });

  function compact(n: number): string {
    return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n);
  }

  function money(value: number): string {
    return `$${value.toFixed(4)}`;
  }
</script>

<footer class="footer">
  <div class="column inner">
    <span class="mode" data-tone={mode.tone}>{mode.label}</span>

    <div class="profile">
      <span class="dot" style={`background: var(--c-${color})`}></span>
      <span class="name">{profile?.name ?? "no profile"}</span>
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

    <div class="usage number" title={totalText}>
      {#if usage}
        ↑{compact(usage.tokensIn)} ↓{compact(usage.tokensOut)}{#if usage.cost !== null}{` · ${money(usage.cost)}`}{/if}{#if provider}{` · ${provider}`}{/if}
      {/if}
    </div>

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
      {:else}
        <button
          type="button"
          class="follow"
          class:on={tracking()}
          title="Follow output / jump to present"
          onclick={followOutput}
        >
          <Icon name="follow" size={13} />
          <Kbd binding={keybinds.follow?.[0]} />
        </button>
      {/if}

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
    </div>

    {#if !sidebar}
      <button
        type="button"
        class="web"
        data-state={web}
        title={webTitle}
        disabled={config?.toolCalling === false}
        onclick={cycleWeb}
      >
        {web === "off" ? "WEB OFF" : web === "on" ? "WEB ON" : "WEB"}
      </button>
    {/if}
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
    gap: var(--sp-2);
    height: 36px;
    font-size: var(--fs-sm);
  }

  .mode {
    display: none;
  }

  .web {
    margin-left: auto;
    padding: 0 6px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    font: inherit;
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.04em;
    cursor: default;
  }

  .web[data-state="on"] {
    background: var(--c-purple, var(--accent));
    color: var(--bg);
  }

  .web[data-state="off"] {
    text-decoration: line-through;
  }

  .profile {
    display: inline-flex;
    align-items: center;
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

  .usage {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: var(--text-tertiary);
    white-space: nowrap;
    text-overflow: ellipsis;
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

  .sticky {
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

  .sticky.on {
    color: var(--accent);
  }

  .follow {
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

  .follow.on {
    color: var(--accent);
  }
</style>
