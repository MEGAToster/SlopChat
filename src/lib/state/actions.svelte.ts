import { tick } from "svelte";

import { messagesViewport, focusComposer } from "../dom";
import { hidePanel, resizePanel, toggleSticky } from "../ipc";
import type { Action } from "../keymap/dispatch";
import {
  isStreaming,
  lastUserMessage,
  nextChat,
  pinChat,
  prevChat,
  regenerate,
  removeChat,
  reportError,
  stop,
  submit,
} from "./chat.svelte";
import {
  activate as paletteActivate,
  close as paletteClose,
  open as paletteOpen,
} from "./palette.svelte";
import {
  closePanel as settingsPanelClose,
  isOpen as settingsPanelIsOpen,
} from "./settingsPanel.svelte";
import {
  activeProfile,
  beginEdit,
  clampSelection,
  cycleProfile,
  editingId,
  endEdit,
  isExpanded,
  keyContext,
  measurePanel,
  modelOverride,
  moveSelection,
  openChatWithProfile,
  pendingDelete,
  schedulePreview,
  selectActive,
  selectedChatId,
  setContext,
  setFiltering,
  setMode,
  setPendingDelete,
  syncProfileFromActive,
} from "./ui.svelte";

export type TextRef = {
  get: () => string;
  set: (value: string) => void;
};

export function createActions(text: TextRef) {
  async function handleSend(): Promise<void> {
    const profile = activeProfile();
    if (!profile) {
      reportError("No profile configured in settings.json");
      return;
    }
    const options = {
      profileName: profile.name,
      model: modelOverride() ?? profile.model,
      system: profile.system,
      temperature: profile.temperature,
      useContext: profile.useContext,
    };
    const editing = editingId();
    const value = text.get();
    text.set("");
    if (editing) {
      endEdit();
      await regenerate(editing, value, options);
    } else {
      await submit(value, options);
    }
  }

  function startEdit(): void {
    const last = lastUserMessage();
    if (!last?.id) return;
    text.set(last.content);
    beginEdit(last.id);
    focusComposer();
  }

  function cancelEdit(): void {
    endEdit();
    text.set("");
    focusComposer();
  }

  async function toggleSidebar(): Promise<void> {
    if (isExpanded()) {
      setFiltering(false);
      setContext("composer");
      await setMode("compact");
      await tick();
      measurePanel();
      focusComposer();
    } else {
      (document.activeElement as HTMLElement | null)?.blur();
      await setMode("expanded");
      selectActive();
      setFiltering(false);
      setContext("sidebar");
    }
  }

  function scrollMessages(direction: number): void {
    const viewport = messagesViewport();
    if (viewport) viewport.scrollTop += direction * viewport.clientHeight * 0.5;
  }

  async function openSelected(): Promise<void> {
    const id = selectedChatId();
    if (id) await openChatWithProfile(id);
    setFiltering(false);
    setContext("composer");
    await tick();
    focusComposer();
  }

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
    if (id) void removeChat(id).then(clampSelection);
  }

  function cancelDelete(): void {
    setPendingDelete(null);
  }

  function escChain(): void {
    if (isStreaming()) {
      void stop();
      return;
    }
    if (keyContext() === "sidebar") {
      setContext("composer");
      focusComposer();
      return;
    }
    if (isExpanded()) {
      void setMode("compact").then(() => tick()).then(measurePanel);
      return;
    }
    void hidePanel();
  }

  function perform(action: Action): void {
    switch (action) {
      case "palette":
        void openPalette();
        break;
      case "stop":
        void stop();
        break;
      case "scrollUp":
        scrollMessages(-1);
        break;
      case "scrollDown":
        scrollMessages(1);
        break;
      case "sticky":
        void toggleSticky();
        break;
      case "send":
        void handleSend();
        break;
      case "cycleProfile":
        cycleProfile();
        break;
      case "toggleSidebar":
        void toggleSidebar();
        break;
      case "editLast":
        startEdit();
        break;
      case "prevChat":
        void prevChat().then(syncProfileFromActive);
        break;
      case "nextChat":
        void nextChat().then(syncProfileFromActive);
        break;
      case "cancelEdit":
        cancelEdit();
        break;
      case "sidebarDown":
        moveSelection(1);
        schedulePreview();
        break;
      case "sidebarUp":
        moveSelection(-1);
        schedulePreview();
        break;
      case "openChat":
        void openSelected();
        break;
      case "pinChat": {
        const id = selectedChatId();
        if (id) void pinChat(id);
        break;
      }
      case "deleteChat": {
        const id = selectedChatId();
        if (id) setPendingDelete(id);
        break;
      }
      case "filterChats":
        if (!isExpanded()) void setMode("expanded");
        setContext("sidebar");
        setFiltering(true);
        break;
    }
  }

  function paletteActivateAndClose(): void {
    void paletteActivate()
      .then((shouldClose) => {
        if (shouldClose) closePalette();
      })
      .catch(closePalette);
  }

  return {
    handleSend,
    startEdit,
    cancelEdit,
    toggleSidebar,
    openSelected,
    openPalette,
    closePalette,
    closeSettings,
    closeFilter,
    confirmDelete,
    cancelDelete,
    escChain,
    perform,
    paletteActivateAndClose,
  };
}
