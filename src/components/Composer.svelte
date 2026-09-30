<script lang="ts">
  import { setSuppressHide } from "../lib/ipc";
  import { addFiles, pending, removePending } from "../lib/state/attachments.svelte";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let {
    value = $bindable(),
    element = $bindable(null),
    placeholder = "Ask anything",
    onfocus,
    editing = false,
  }: {
    value: string;
    element?: HTMLTextAreaElement | null;
    placeholder?: string;
    onfocus?: () => void;
    editing?: boolean;
  } = $props();

  const MAX_HEIGHT = 8 * 21;

  let fileInput = $state<HTMLInputElement | null>(null);
  let dragging = $state(false);

  $effect(() => {
    void value;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${Math.min(element.scrollHeight, MAX_HEIGHT)}px`;
  });

  function onDrop(event: DragEvent) {
    event.preventDefault();
    dragging = false;
    if (event.dataTransfer?.files.length) addFiles(event.dataTransfer.files);
  }

  function onPaste(event: ClipboardEvent) {
    const files = event.clipboardData?.files;
    if (files && files.length > 0) {
      event.preventDefault();
      addFiles(files);
    }
  }

  async function pick() {
    await setSuppressHide(true);
    fileInput?.click();
  }

  function onPicked() {
    void setSuppressHide(false);
    if (fileInput?.files?.length) addFiles(fileInput.files);
    if (fileInput) fileInput.value = "";
  }
</script>

<div
  class="composer-box column"
  class:editing
  class:dragging
  role="presentation"
  ondragover={(event) => {
    event.preventDefault();
    dragging = true;
  }}
  ondragleave={() => (dragging = false)}
  ondrop={onDrop}
>
  {#if editing}<span class="editing-label">Editing message</span>{/if}

  {#if pending().length > 0}
    <div class="chips">
      {#each pending() as file (file.id)}
        <span class="chip">
          {#if file.mime.startsWith("image/")}
            <img src={file.dataUrl} alt="" />
          {:else}
            <Icon name="attach" size={12} />
          {/if}
          <span class="chip-name">{file.name}</span>
          <button type="button" aria-label="Remove" onclick={() => removePending(file.id)}>
            <Icon name="trash" size={12} />
          </button>
        </span>
      {/each}
    </div>
  {/if}

  <div class="row">
    <button type="button" class="attach" title="Attach" onclick={() => void pick()}>
      <Icon name="attach" size={15} />
    </button>
    <textarea
      bind:this={element}
      bind:value
      class="input"
      {placeholder}
      rows="1"
      spellcheck="false"
      onpaste={onPaste}
      onfocus={() => onfocus?.()}
    ></textarea>
    <span class="enter" class:dim={!value.trim()}><Kbd binding="Enter" /></span>
  </div>

  <input bind:this={fileInput} class="hidden" type="file" multiple onchange={onPicked} oncancel={onPicked} />
</div>

<style>
  .composer-box {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    width: 100%;
    border-radius: var(--radius-md);
  }

  .composer-box.dragging {
    outline: 1px dashed var(--accent);
    outline-offset: 4px;
  }

  .composer-box.editing {
    padding-left: var(--sp-2);
    border-left: 2px solid var(--accent);
  }

  .editing-label {
    color: var(--text-tertiary);
    font-size: var(--fs-xs);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    max-width: 200px;
    padding: 3px 6px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--text-secondary);
    font-size: var(--fs-xs);
  }

  .chip img {
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    object-fit: cover;
  }

  .chip-name {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .chip button {
    display: inline-flex;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .chip button:hover {
    color: var(--danger);
  }

  .row {
    display: flex;
    align-items: flex-end;
    gap: var(--sp-2);
  }

  .attach {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .attach:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .input {
    flex: 1;
    max-height: 168px;
    resize: none;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: var(--fs-lg);
    line-height: 1.4;
  }

  .input::placeholder {
    color: var(--text-tertiary);
  }

  .enter {
    padding-bottom: 2px;
  }

  .enter.dim {
    opacity: 0.4;
  }

  .hidden {
    display: none;
  }
</style>
