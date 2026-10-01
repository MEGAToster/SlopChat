import { tick } from "svelte";

import { focusComposer } from "../dom";
import { hidePanel, resizePanel } from "../ipc";
import { removeChat } from "./chat.svelte";
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

  function closePalette(): void {
    paletteClose();
    setContext("composer");
    if (settingsPanelIsOpen()) return;
    if (!isExpanded()) void tick().then(measurePanel);
    focusComposer();
  }

  function closeSettings(): void {
    settingsPanelClose();
    if (!isExpanded()) void tick().then(measurePanel);
    focusComposer();
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
