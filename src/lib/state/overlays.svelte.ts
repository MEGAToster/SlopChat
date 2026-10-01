import { tick } from "svelte";

import { focusComposer } from "../dom";
import { hidePanel, resizePanel } from "../ipc";
import { removeChat } from "./chatHistory.svelte";
import {
  close as paletteClose,
  isOpen as paletteIsOpen,
  open as paletteOpen,
} from "./palette.svelte";
import {
  closePanel as settingsPanelClose,
  isOpen as settingsPanelIsOpen,
} from "./settingsPanel.svelte";
import {
  clampSelection,
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

// Overlays are absolutely positioned inside the panel, so while one is open the
// compact height must not follow the (hidden) thread underneath it.
export function measurePanel(): void {
  if (isExpanded()) return;
  if (paletteIsOpen() || settingsPanelIsOpen()) {
    void resizePanel(OVERLAY_HEIGHT);
    return;
  }
  const content = document.querySelector<HTMLElement>(".content");
  if (content) void resizePanel(Math.ceil(content.getBoundingClientRect().height));
}

export function createOverlays() {
  async function openPalette(): Promise<void> {
    if (!isExpanded()) void resizePanel(OVERLAY_HEIGHT);
    setContext("palette");
    await paletteOpen();
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
    setContext("composer");
    focusComposer();
  }

  function confirmDelete(): void {
    const id = pendingDelete();
    setPendingDelete(null);
    if (id) {
      void removeChat(id).then(() => {
        clampSelection();
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

  function openSidebar(): void {
    void setMode("expanded");
    selectActive();
    setFiltering(false);
    setContext("sidebar");
  }

  return {
    openPalette,
    closePalette,
    closeSettings,
    closeFilter,
    confirmDelete,
    cancelDelete,
    escChain,
    openSidebar,
  };
}
