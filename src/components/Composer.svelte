<script lang="ts">
  import { setSuppressHide } from "../lib/ipc";
  import { createComposerPaste } from "../lib/composerPaste";
  import { addFiles, pending, removePending } from "../lib/state/attachments.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { formatBytes } from "../lib/text";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let {
    value = $bindable(),
    onfocus,
    editing = false,
  }: {
    value: string;
    onfocus?: () => void;
    editing?: boolean;
  } = $props();

  // Measuring a very large value forces a full textarea reflow; past this size
  // just pin to the max height instead.
  const FIT_MAX_CHARS = 20_000;

  let element = $state<HTMLTextAreaElement | null>(null);
  let fileInput = $state<HTMLInputElement | null>(null);
  let dragging = $state(false);

  function fit(element: HTMLTextAreaElement): void {
    if (element.clientWidth === 0) return;
    // The line height scales with the font setting, so measure it instead of
    // assuming a pixel value.
    const lineHeight = parseFloat(getComputedStyle(element).lineHeight) || 21;
    const maxHeight = 8 * lineHeight;
    if (element.value.length > FIT_MAX_CHARS) {
      element.style.height = `${maxHeight}px`;
      return;
    }
    // Collapse first so scrollHeight reflects the content, not the last height.
    element.style.height = "0px";
    element.style.height = `${Math.max(lineHeight, Math.min(element.scrollHeight, maxHeight))}px`;
  }

  $effect(() => {
    void value;
    if (element) fit(element);
  });

  // The panel is built hidden and shown later; refit when the width settles.
  $effect(() => {
    const target = element;
    if (!target) return;
    let width = target.clientWidth;
    const observer = new ResizeObserver(() => {
      if (target.clientWidth === width) return;
      width = target.clientWidth;
      fit(target);
    });
    observer.observe(target);
    return () => observer.disconnect();
  });

  function onDrop(event: DragEvent) {
    event.preventDefault();
    dragging = false;
    if (event.dataTransfer?.files.length) addFiles(event.dataTransfer.files);
  }

  // Moving between the box and its textarea fires dragleave; only clear when
  // the pointer actually leaves the box.
  function onDragLeave(event: DragEvent) {
    const next = event.relatedTarget as Node | null;
    if (next && (event.currentTarget as HTMLElement).contains(next)) return;
    dragging = false;
  }

  let dialogOpen = $state(false);

  const paste = createComposerPaste({
    element: () => element,
    getValue: () => value,
    setValue: (next) => (value = next),
    fit,
    pasteAsFileChars: () => settings().pasteAsFileChars,
  });

  async function pick() {
    dialogOpen = true;
    await setSuppressHide(true);
    fileInput?.click();
  }

  function onPicked() {
    dialogOpen = false;
    void setSuppressHide(false);
    if (fileInput?.files?.length) addFiles(fileInput.files);
    if (fileInput) fileInput.value = "";
  }

  // WebKitGTK may not fire `cancel`; the panel regains focus once the chooser
  // closes, so restore the layer then too.
  $effect(() => {
    const restore = () => {
      if (!dialogOpen) return;
      dialogOpen = false;
      void setSuppressHide(false);
    };
    window.addEventListener("focus", restore);
    return () => window.removeEventListener("focus", restore);
  });
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
  ondragleave={onDragLeave}
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
          {#if !file.mime.startsWith("image/")}
            <span class="chip-size">· {formatBytes(file.size)}</span>
          {/if}
          <button type="button" aria-label="Remove" onclick={() => removePending(file.id)}>
            <Icon name="trash" size={12} />
          </button>
        </span>
      {/each}
    </div>
  {/if}

  <div class="row">
    <button type="button" class="attach" title="Attach" aria-label="Attach" onclick={() => void pick()}>
      <Icon name="attach" size={15} />
    </button>
    <textarea
      bind:this={element}
      bind:value
      class="input"
      placeholder="Ask anything"
      rows="1"
      spellcheck="false"
      onpaste={paste.onPaste}
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

  .chip-size {
    flex: none;
    color: var(--text-tertiary);
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
