import { tick } from "svelte";

import { pastedFiles } from "./clipboard";
import { readClipboard } from "./ipc";
import { shouldPasteAsFile } from "./paste";
import { addFiles } from "./state/attachments.svelte";
import type { ClipboardContent } from "./types";

type Deps = {
  element: () => HTMLTextAreaElement | null;
  getValue: () => string;
  setValue: (value: string) => void;
  fit: (element: HTMLTextAreaElement) => void;
  pasteAsFileChars: () => number;
};

// Composer clipboard handling, kept out of the component so its markup stays small.
export function createComposerPaste(deps: Deps) {
  let pasteCounter = 0;
  // WebKitGTK leaves clipboardData empty on Linux only; elsewhere the DOM path
  // is authoritative and preventing the default paste would drop content.
  const nativeClipboard = navigator.userAgent.includes("Linux");

  function addPastedText(text: string) {
    pasteCounter += 1;
    addFiles([new File([text], `paste-${pasteCounter}.txt`, { type: "text/plain" })]);
  }

  async function insertAtCaret(text: string) {
    const target = deps.element();
    if (!target) return;
    const current = deps.getValue();
    const start = target.selectionStart ?? current.length;
    const end = target.selectionEnd ?? start;
    deps.setValue(current.slice(0, start) + text + current.slice(end));
    await tick();
    const caret = start + text.length;
    target.setSelectionRange(caret, caret);
    deps.fit(target);
  }

  async function pngFile(pngBase64: string): Promise<File | null> {
    try {
      const blob = await (await fetch(`data:image/png;base64,${pngBase64}`)).blob();
      return new File([blob], `pasted-${Date.now()}.png`, { type: "image/png" });
    } catch {
      return null;
    }
  }

  async function pasteFromClipboard() {
    let content: ClipboardContent;
    try {
      content = await readClipboard();
    } catch {
      return;
    }
    if (content.text) {
      if (shouldPasteAsFile(content.text, deps.pasteAsFileChars())) addPastedText(content.text);
      else await insertAtCaret(content.text);
      return;
    }
    if (content.image) {
      const file = await pngFile(content.image.pngBase64);
      if (file) addFiles([file]);
    }
  }

  function onPaste(event: ClipboardEvent) {
    const clipboard = event.clipboardData;
    const files = pastedFiles(clipboard);
    if (files.length > 0) {
      event.preventDefault();
      addFiles(files);
      return;
    }
    const text = clipboard?.getData("text/plain") ?? "";
    if (text) {
      if (shouldPasteAsFile(text, deps.pasteAsFileChars())) {
        event.preventDefault();
        addPastedText(text);
      }
      return;
    }
    // A non-text payload the platform can paste itself (rich text, image).
    if (clipboard && clipboard.types.length > 0) return;
    if (!nativeClipboard) return;
    event.preventDefault();
    void pasteFromClipboard();
  }

  return { onPaste };
}
