import { tick } from "svelte";

import { focusComposer } from "../dom";
import { hidePanel, resizeOverlay, resizePanel } from "../ipc";
import { ResizeGate, type ResizeMode } from "../resizeGate";
import { isNavigating, removeChat } from "./chatHistory.svelte";
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
  renamingId,
  selectActive,
  setContext,
  setMode,
  setPendingDelete,
} from "./ui.svelte";
import { setFiltering } from "./chatFilter.svelte";

// Overlays are absolutely positioned inside the panel, so while one is open the
// compact height must not follow the (hidden) thread underneath it.
//
// A native resize changes the webview viewport, which the App's ResizeObserver
// feeds straight back into measurePanel; unguarded that self-sustains and locks
// the GTK main loop. Coalesce bursts, skip sub-pixel echoes, and never have more
// than one resize IPC in flight.
const RESIZE_DEBOUNCE_MS = 50;

const resizeGate = new ResizeGate();
let pending: { mode: ResizeMode; height: number } | null = null;
let flushTimer: ReturnType<typeof setTimeout> | undefined;
let inFlight = false;

function scheduleResize(mode: ResizeMode, height: number): void {
  if (!resizeGate.accept(mode, height)) return;
  pending = { mode, height };
  clearTimeout(flushTimer);
  flushTimer = setTimeout(flushResize, RESIZE_DEBOUNCE_MS);
}

function flushResize(): void {
  if (inFlight || !pending) return;
  const { mode, height } = pending;
  pending = null;
  resizeGate.applied(mode, height);
  inFlight = true;
  const request = mode === "overlay" ? resizeOverlay(height) : resizePanel(height);
  void request.finally(() => {
    inFlight = false;
    if (pending) flushResize();
  });
}

export function measurePanel(): void {
  // Defer while a history walk is in flight; the settled call re-measures once.
  if (isNavigating() || isExpanded()) return;
  if (paletteIsOpen() || settingsPanelIsOpen()) {
    scheduleResize("overlay", OVERLAY_HEIGHT);
    return;
  }
  const content = document.querySelector<HTMLElement>(".content");
  if (content) scheduleResize("panel", Math.ceil(content.getBoundingClientRect().height));
}

export function createOverlays() {
  async function openPalette(): Promise<void> {
    if (!isExpanded()) scheduleResize("overlay", OVERLAY_HEIGHT);
    setContext("palette");
    await paletteOpen();
    await tick();
  }

  async function openModels(): Promise<void> {
    if (!isExpanded()) scheduleResize("overlay", OVERLAY_HEIGHT);
    setContext("palette");
    await paletteOpenModels();
    await tick();
  }

  // Focus must wait for the overlay to unmount (while open, the composer sits
  // under `.panel.overlay .layout`, i.e. visibility: hidden, and can't focus)
  // and for the panel resize that follows, which can steal focus.
  function restoreComposerFocus(): void {
    if (renamingId() !== null) return;
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

  function setSidebar(focusComposer: boolean): void {
    void setMode("expanded");
    setFiltering(false);
    selectActive();
    setContext(focusComposer ? "composer" : "sidebar");
  }

  // Expand and focus the sidebar.
  function openSidebar(): void {
    setSidebar(false);
  }

  // Expand the sidebar but keep focus in the composer.
  function expandSidebar(): void {
    setSidebar(true);
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
