<script lang="ts">
  import Frame from "./Frame.svelte";
  import Icon from "./Icon.svelte";

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

  let { message, kind = "info" }: { message: string; kind?: ToastKind } = $props();
</script>

<div class="toast" class:warning={kind === "warning"} class:success={kind === "success"}>
  <Frame title={titles[kind]} tone={tones[kind]}>
    <span class="icon" class:info={kind === "info"}><Icon name={icons[kind]} size={14} /></span>
    <span>{message}</span>
  </Frame>
</div>

<style>
  .toast {
    position: absolute;
    left: 50%;
    bottom: calc(36px + var(--sp-3));
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
    animation: rise-in var(--dur) var(--ease);
  }

  .icon {
    display: inline-flex;
    flex: none;
    color: var(--danger);
  }

  .toast.warning .icon {
    color: var(--warning);
  }

  .toast.success .icon {
    color: var(--success);
  }

  .icon.info {
    color: var(--accent);
  }
</style>
