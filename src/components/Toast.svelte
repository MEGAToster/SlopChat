<script lang="ts">
  import Frame from "./Frame.svelte";
  import Icon from "./Icon.svelte";
  import { clearNotice } from "../lib/state/ui.svelte";

  type ToastKind = "danger" | "warning" | "success" | "info";

  const titles: Record<ToastKind, string> = {
    danger: "error",
    warning: "warning",
    success: "done",
    info: "notice",
  };
  const tones: Record<ToastKind, string> = {
    danger: "red",
    warning: "yellow",
    success: "green",
    info: "accent",
  };
  const icons: Record<ToastKind, string> = {
    danger: "alert",
    warning: "alert",
    success: "check",
    info: "spark",
  };

  let {
    message,
    kind = "info",
    pinned = false,
  }: { message: string; kind?: ToastKind; pinned?: boolean } = $props();
</script>

<div
  class="toast"
  class:pinned
  data-kind={kind}
  role={kind === "danger" ? "alert" : "status"}
  aria-live={kind === "danger" ? "assertive" : "polite"}
  onclick={pinned ? undefined : clearNotice}
>
  <Frame title={titles[kind]} tone={tones[kind]}>
    <span class="toast-body">
      <span class="icon"><Icon name={icons[kind]} size={14} /></span>
      <span class="text">{message}</span>
    </span>
  </Frame>
</div>

<style>
  .toast {
    position: absolute;
    left: 50%;
    bottom: calc(var(--footer-height) + var(--sp-3));
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    max-width: 80%;
    padding: var(--sp-2) var(--sp-3);
    transform: translateX(-50%);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    background: var(--bg-solid);
    box-shadow: var(--shadow-overlay);
    color: var(--text-secondary);
    font-size: var(--fs-sm);
    overflow-wrap: anywhere;
    cursor: pointer;
    animation: rise-in var(--dur) var(--ease);
  }

  .toast.pinned {
    top: var(--sp-3);
    bottom: auto;
    cursor: default;
  }

  /* A row even inside the TUI frame body, which is a column. */
  .toast-body {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: var(--sp-2);
    min-width: 0;
  }

  .text {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 4;
    line-clamp: 4;
  }

  .icon {
    display: inline-flex;
    flex: none;
    color: var(--danger);
  }

  .toast[data-kind="warning"] .icon {
    color: var(--warning);
  }

  .toast[data-kind="success"] .icon {
    color: var(--success);
  }

  .toast[data-kind="info"] .icon {
    color: var(--accent);
  }
</style>
