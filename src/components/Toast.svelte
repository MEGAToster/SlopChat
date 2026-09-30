<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    message,
    kind = "danger",
  }: { message: string; kind?: "danger" | "warning" | "success" } = $props();
</script>

<div class="toast" class:warning={kind === "warning"} class:success={kind === "success"}>
  <span class="icon"><Icon name={kind === "success" ? "check" : "alert"} size={14} /></span>
  <span>{message}</span>
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
</style>
