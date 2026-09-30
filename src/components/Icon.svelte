<script lang="ts">
  let { name, size = 16 }: { name: string; size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === "search"}
    <circle cx="11" cy="11" r="7" />
    <path d="m20 20-3.6-3.6" />
  {:else if name === "action"}
    <path d="M13 2 4 14h6l-1 8 9-12h-6z" />
  {:else if name === "chat"}
    <path d="M21 15a2 2 0 0 1-2 2H8l-4 4V5a2 2 0 0 1 2-2h13a2 2 0 0 1 2 2z" />
  {:else if name === "model"}
    <rect x="6" y="6" width="12" height="12" rx="2" />
    <path d="M9 2v2M15 2v2M9 20v2M15 20v2M2 9h2M2 15h2M20 9h2M20 15h2" />
  {:else if name === "profile"}
    <circle cx="12" cy="8" r="4" />
    <path d="M5 21a7 7 0 0 1 14 0" />
  {:else if name === "theme"}
    <circle cx="12" cy="12" r="4" />
    <path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M18.4 5.6 17 7M7 17l-1.4 1.4" />
  {:else if name === "provider"}
    <path d="M12 3 3 8v8l9 5 9-5V8z" />
    <path d="M3 8l9 5 9-5M12 13v8" />
  {:else if name === "pin"}
    <path d="M12 17v5M9 3h6l-1 6 3 3H7l3-3z" />
  {:else if name === "trash"}
    <path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13" />
  {:else if name === "copy"}
    <rect x="9" y="9" width="11" height="11" rx="2" />
    <path d="M5 15V5a2 2 0 0 1 2-2h8" />
  {:else if name === "check"}
    <path d="m5 13 4 4 10-10" />
  {:else if name === "alert"}
    <path d="M12 3 2 20h20z" />
    <path d="M12 10v4M12 17.5v.5" />
  {:else if name === "stop"}
    <rect x="6" y="6" width="12" height="12" rx="2" />
  {:else if name === "spark"}
    <path d="M12 3v4M12 17v4M3 12h4M17 12h4M6 6l2.5 2.5M15.5 15.5 18 18M18 6l-2.5 2.5M8.5 15.5 6 18" />
  {:else if name === "enter"}
    <path d="M9 10 4 15l5 5" />
    <path d="M20 4v7a4 4 0 0 1-4 4H4" />
  {/if}
</svg>
