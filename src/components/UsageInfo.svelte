<script lang="ts">
  import type { Usage } from "../lib/types";
  import { messages } from "../lib/state/chatStore.svelte";
  import { money, usageLine } from "../lib/usageInfo";

  let { usage, apiProvider }: { usage: Usage | null; apiProvider: string } = $props();

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
</script>

<div class="usage number" title={totalText}>
  {#if usage}{usageLine(usage, apiProvider)}{/if}
</div>

<style>
  .usage {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: var(--text-tertiary);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
</style>
