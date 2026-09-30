<script lang="ts">
  import { lastUsage, messages } from "../lib/state/chat.svelte";

  function formatCost(cost: number): string {
    return `$${cost.toFixed(4)}`;
  }

  const last = $derived(lastUsage());
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
</script>

<div class="status">
  {#if last}
    <span class="last">
      ↑{last.tokensIn} ↓{last.tokensOut}{#if last.cost !== null} · {formatCost(last.cost)}{/if}
    </span>
    <span class="total">
      total ↑{total.tokensIn} ↓{total.tokensOut}{#if total.hasCost} · {formatCost(total.cost)}{/if}
    </span>
  {/if}
</div>
