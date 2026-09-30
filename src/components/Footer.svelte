<script lang="ts">
  import { profileColor } from "../lib/profileColor";
  import { isStreamingHere, lastUsage, messages } from "../lib/state/chat.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import {
    activeProfile,
    activeProfileIndex,
    editingId,
    keyContext,
    modelOverride,
  } from "../lib/state/ui.svelte";
  import { toggleSticky } from "../lib/ipc";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let { sticky = false }: { sticky?: boolean } = $props();

  const profile = $derived(activeProfile());
  const color = $derived(profileColor(profile, activeProfileIndex()));
  const fullModel = $derived(modelOverride() ?? profile?.model ?? "");
  const model = $derived(fullModel.split("/").pop() ?? "");
  const usage = $derived(lastUsage());
  const provider = $derived(usage?.provider ?? fullModel.split("/")[0] ?? "");
  const keybinds = $derived(settings().keybinds);
  const sidebar = $derived(keyContext() === "sidebar");
  const editing = $derived(editingId() !== null);

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
        <span><Kbd binding={keybinds.palette?.[0]} /> Commands</span>
      {/if}

      <button type="button" class="sticky" class:on={sticky} onclick={() => void toggleSticky()}>
        <Icon name="lock" size={13} />
        <span>Sticky</span>
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
    gap: var(--sp-3);
    height: 36px;
    font-size: var(--fs-sm);
  }

  .mode {
    display: none;
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

  .usage {
    flex: 1;
    color: var(--text-tertiary);
  }

  .hints {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
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
</style>
