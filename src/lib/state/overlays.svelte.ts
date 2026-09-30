import { tick } from "svelte";

import { focusComposer } from "../dom";
import { hidePanel, resizePanel } from "../ipc";
import { isStreaming, removeChat, stop } from "./chat.svelte";
import { close as paletteClose, open as paletteOpen } from "./palette.svelte";
import {
  closePanel as settingsPanelClose,
  isOpen as settingsPanelIsOpen,
} from "./settingsPanel.svelte";
import {
  clampSelection,
  endEdit,
  isExpanded,
  measurePanel,
  pendingDelete,
  selectActive,
  setContext,
  setFiltering,
  setMode,
  setPendingDelete,
} from "./ui.svelte";

export function createOverlays() {
  async function openPalette(): Promise<void> {
    if (!isExpanded()) void resizePanel(480);
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
    if (isStreaming()) {
      void stop();
      return;
    }
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
