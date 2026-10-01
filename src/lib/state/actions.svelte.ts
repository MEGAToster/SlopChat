import { tick } from "svelte";

import { focusComposer, messagesViewport } from "../dom";
import { toggleSticky } from "../ipc";
import type { Action } from "../keymap/dispatch";
import {
  lastUserMessage,
  nextChat,
  pinChat,
  prevChat,
  regenerate,
  reportError,
  stop,
  submit,
  toggleWeb,
  webOverride,
} from "./chat.svelte";
import { createOverlays } from "./overlays.svelte";
import { activate as paletteActivate } from "./palette.svelte";
import {
  activeProfile,
  beginEdit,
  cycleProfile,
  editingId,
  endEdit,
  isExpanded,
  modelOverride,
  moveSelection,
  openChatWithProfile,
  schedulePreview,
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
  const overlays = createOverlays();

  async function handleSend(): Promise<void> {
    const profile = activeProfile();
    if (!profile) {
      reportError("No profile configured in settings.json");
      return;
    }
    const override = webOverride();
    const webSearch =
      override === "off" ? "off" : override === "on" ? "always" : profile.webSearch;
    const options = {
      profileName: profile.name,
      model: modelOverride() ?? profile.model,
      system: profile.system,
      temperature: profile.temperature,
      useContext: profile.useContext,
      tools: profile.tools,
      webSearch,
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
      focusComposer();
    } else {
      overlays.openSidebar();
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

  function perform(action: Action): void {
    switch (action) {
      case "palette":
        void overlays.openPalette();
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
      case "webToggle":
        toggleWeb();
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
        void prevChat().then(() => {
          syncProfileFromActive();
          endEdit();
        });
        break;
      case "nextChat":
        void nextChat().then(() => {
          syncProfileFromActive();
          endEdit();
        });
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
        if (shouldClose) overlays.closePalette();
      })
      .catch(overlays.closePalette);
  }

  return {
    handleSend,
    startEdit,
    cancelEdit,
    toggleSidebar,
    openSelected,
    perform,
    paletteActivateAndClose,
    ...overlays,
  };
}
