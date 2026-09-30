<script lang="ts">
  let {
    value = $bindable(),
    element = $bindable(null),
    disabled = false,
    placeholder = "Ask anything",
    onfocus,
  }: {
    value: string;
    element?: HTMLTextAreaElement | null;
    disabled?: boolean;
    placeholder?: string;
    onfocus?: () => void;
  } = $props();
</script>

<textarea
  bind:this={element}
  bind:value
  class="composer"
  {placeholder}
  {disabled}
  rows="1"
  spellcheck="false"
  onfocus={() => onfocus?.()}
></textarea>
