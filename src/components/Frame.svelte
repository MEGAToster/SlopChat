<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title = "",
    hint = "",
    focused = false,
    tone = "accent",
    count = "",
    children,
  }: {
    title?: string;
    hint?: string;
    focused?: boolean;
    tone?: string;
    count?: string;
    children?: Snippet;
  } = $props();

  const toneVar = $derived(tone === "accent" ? "var(--accent)" : `var(--c-${tone})`);
</script>

<section class="frame" class:focused style={`--frame-tone: ${toneVar}`}>
  <div class="edge top">
    <span class="corner left"></span>
    <span class="title">{title}</span>
    <span class="line"></span>
    {#if count}<span class="count">{count}</span>{/if}
    <span class="corner right"></span>
  </div>
  <div class="body">{@render children?.()}</div>
  <div class="edge bottom">
    <span class="corner left"></span>
    <span class="line"></span>
    {#if hint}<span class="hint">{hint}</span>{/if}
    <span class="corner right"></span>
  </div>
</section>

<style>
  /* Minimal style: the frame is invisible. TUI decoration lives in tui.css. */
  .frame,
  .body {
    display: contents;
  }

  .edge {
    display: none;
  }
</style>
