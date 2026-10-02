import { tick } from "svelte";

import { focusComposer } from "../dom";
import { hidePanel, resizeOverlay, resizePanel } from "../ipc";
import { removeChat } from "./chatHistory.svelte";
import {
  close as paletteClose,
  isOpen as paletteIsOpen,
  open as paletteOpen,
  openModels as paletteOpenModels,
} from "./palette.svelte";
import {
  closePanel as settingsPanelClose,
  isOpen as settingsPanelIsOpen,
} from "./settingsPanel.svelte";
import {
  endEdit,
  isExpanded,
  OVERLAY_HEIGHT,
  pendingDelete,
  selectActive,
  setContext,
  setFiltering,
  setMode,
  setPendingDelete,
} from "./ui.svelte";

// Natural height of the transcript + composer + footer, so the panel can grow
// with content up to the configured ceiling (the backend clamps it).
function contentHeight(): number {
  const content = document.querySelector<HTMLElement>(".content");
  if (!content) return 0;

  const style = getComputedStyle(content);
  const gap = parseFloat(style.rowGap || style.gap) || 0;
  let total = (parseFloat(style.paddingTop) || 0) + (parseFloat(style.paddingBottom) || 0);
  let blocks = 0;

  const add = (height: number) => {
    if (height <= 0) return;
    if (blocks > 0) total += gap;
    total += height;
    blocks += 1;
  };

  const messages = content.querySelector<HTMLElement>(".messages");
  if (messages) {
    const frame = messages.closest<HTMLElement>(".frame");
    const chrome = frame
      ? frame.getBoundingClientRect().height - messages.getBoundingClientRect().height
      : 0;
    add(messages.scrollHeight + chrome);
  }

  const composer = content.querySelector<HTMLElement>(".composer-box");
  if (composer) add((composer.closest<HTMLElement>(".frame") ?? composer).getBoundingClientRect().height);

  const footer = content.querySelector<HTMLElement>(".footer");
  if (footer) add(footer.getBoundingClientRect().height);

  if (blocks === 0) {
    for (const child of Array.from(content.children) as HTMLElement[]) {
      if (child.classList.contains("toast")) continue;
      add(child.scrollHeight);
    }
  }

  return Math.ceil(total);
}

let lastHeight = -1;

// Overlays are absolutely positioned inside the panel, so while one is open the
// compact height must not follow the (hidden) thread underneath it.
export function measurePanel(): void {
  if (isExpanded()) {
    lastHeight = -1;
    return;
  }
  if (paletteIsOpen() || settingsPanelIsOpen()) {
    lastHeight = -1;
    void resizeOverlay(OVERLAY_HEIGHT);
    return;
  }

  const height = contentHeight();
  if (Math.abs(height - lastHeight) < 2) return;
  lastHeight = height;
  void resizePanel(height);
}

export function createOverlays() {
  async function openPalette(): Promise<void> {
    if (!isExpanded()) void resizeOverlay(OVERLAY_HEIGHT);
    setContext("palette");
    await paletteOpen();
    await tick();
  }

  async function openModels(): Promise<void> {
    if (!isExpanded()) void resizeOverlay(OVERLAY_HEIGHT);
    setContext("palette");
    await paletteOpenModels();
    await tick();
  }

  // Focus must wait for the overlay to unmount (while open, the composer sits
  // under `.panel.overlay .layout`, i.e. visibility: hidden, and can't focus)
  // and for the panel resize that follows, which can steal focus.
  function restoreComposerFocus(): void {
    void tick().then(() => {
      if (!isExpanded()) measurePanel();
      requestAnimationFrame(() => {
        focusComposer();
        setTimeout(focusComposer, 0);
      });
    });
  }

  function closePalette(): void {
    paletteClose();
    setContext("composer");
    if (settingsPanelIsOpen()) return;
    restoreComposerFocus();
  }

  function closeSettings(): void {
    settingsPanelClose();
    restoreComposerFocus();
  }

  function closeFilter(): void {
    setFiltering(false);
    selectActive();
    setContext("composer");
    focusComposer();
  }

  function confirmDelete(): void {
    const id = pendingDelete();
    setPendingDelete(null);
    if (id) {
      void removeChat(id).then(() => {
        selectActive();
        endEdit();
      });
    }
  }

  function cancelDelete(): void {
    setPendingDelete(null);
  }

  function escChain(): void {
    void hidePanel();
  }

  // Expand and focus the sidebar.
  function openSidebar(): void {
    void setMode("expanded");
    setFiltering(false);
    selectActive();
    setContext("sidebar");
  }

  // Expand the sidebar but keep focus in the composer.
  function expandSidebar(): void {
    void setMode("expanded");
    setFiltering(false);
    selectActive();
    setContext("composer");
  }

  return {
    openPalette,
    openModels,
    closePalette,
    closeSettings,
    closeFilter,
    confirmDelete,
    cancelDelete,
    escChain,
    openSidebar,
    expandSidebar,
  };
}
