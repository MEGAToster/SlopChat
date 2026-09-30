<script lang="ts">
  let {
    value = $bindable(),
    element = $bindable(null),
    disabled = false,
    placeholder = "Ask anything",
  }: {
    value: string;
    element?: HTMLTextAreaElement | null;
    disabled?: boolean;
    placeholder?: string;
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
></textarea>
