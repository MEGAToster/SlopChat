import { tick } from "svelte";

import { focusComposer, messagesViewport } from "../dom";
import { toggleSticky } from "../ipc";
import type { Action } from "../keymap/dispatch";
import { regenerate, stop, submit } from "./chat.svelte";
import { nextChat, pinChat, prevChat } from "./chatHistory.svelte";
import { chatState, cycleWeb, lastUserMessage, reportError, webOverride } from "./chatStore.svelte";
import { createOverlays } from "./overlays.svelte";
import { activate as paletteActivate } from "./palette.svelte";
import { openPanel } from "./settingsPanel.svelte";
import { settings } from "./settings.svelte";
import { followOutput } from "./view.svelte";
import { cycleReasoningFor } from "./reasoning.svelte";
import { effectiveWebMode, paidSearchWarning } from "../webSearch";
import {
  activeConfig,
  beginEdit,
  currentSendOptions,
  cycleProfile,
  editingId,
  endEdit,
  isExpanded,
  moveSelection,
  newChatWithProfile,
  notify,
  openChatWithProfile,
  schedulePreview,
  selectActive,
  selectedChatId,
  setContext,
  setFiltering,
  setMode,
  setPendingDelete,
  setRenaming,
  syncProfileFromActive,
} from "./ui.svelte";

type TextRef = {
  get: () => string;
  set: (value: string) => void;
};

export function createActions(text: TextRef) {
  const overlays = createOverlays();

  // Deep research is meant to run on free SearXNG; warn once per trigger when
  // it would actually spend paid search credits.
  function warnPaidSearch(model: string): void {
    if (model.startsWith("antigravity/")) return;
    const warning = paidSearchWarning(settings().search);
    if (warning) notify(warning, "warning");
  }

  async function handleSend(): Promise<void> {
    if (chatState.sending) {
      notify("Still answering — Ctrl+Q to stop", "info");
      return;
    }

    const options = currentSendOptions();
    if (!options) {
      reportError("No profile configured in settings.json");
      return;
    }
    if (options.webSearch === "deep") warnPaidSearch(options.model);

    const editing = editingId();
    const value = text.get();
    chatState.sending = true;
    // Clear straight away so the field is empty while the answer streams and
    // new typing isn't wiped when submit resolves.
    text.set("");
    try {
      const sent = editing
        ? await regenerate(editing, value, options)
        : await submit(value, options);
      if (!sent) text.set(value);
      else if (editing) endEdit();
    } catch (cause) {
      if (text.get().length === 0) text.set(value);
      reportError(String(cause));
    } finally {
      chatState.sending = false;
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
      overlays.expandSidebar();
    }
  }

  function scrollMessages(direction: number): void {
    const viewport = messagesViewport();
    if (viewport) viewport.scrollTop += direction * viewport.clientHeight * 0.5;
  }

  async function openSelected(): Promise<void> {
    const id = selectedChatId();
    if (id) await openChatWithProfile(id);
    else await newChatWithProfile();
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
      case "webToggle": {
        const config = activeConfig();
        if (config?.toolCalling === false) break;
        const mode = effectiveWebMode(true, webOverride(), config?.webSearch);
        cycleWeb(mode);
        // auto → deep is the one step that turns deep research on.
        if (mode === "auto") warnPaidSearch(config?.model ?? "");
        break;
      }
      case "reasoning": {
        const config = activeConfig();
        if (config) void cycleReasoningFor(config.model);
        break;
      }
      case "settings":
        openPanel();
        break;
      case "models":
        void overlays.openModels();
        break;
      case "follow":
        followOutput();
        break;
      case "focusSidebar":
        overlays.openSidebar();
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
          selectActive();
          endEdit();
        });
        break;
      case "nextChat":
        void nextChat().then(() => {
          syncProfileFromActive();
          selectActive();
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
      case "renameChat": {
        const id = selectedChatId();
        if (id) setRenaming(id);
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
