import type { Keybinds } from "../types";
import { eventToBinding, matchesAny } from "./parse";

export type Context = "composer" | "sidebar" | "palette" | "filter" | "confirm";

export type Action =
  | "hide"
  | "palette"
  | "paletteClose"
  | "paletteUp"
  | "paletteDown"
  | "paletteRun"
  | "filterClose"
  | "confirmDelete"
  | "cancelDelete"
  | "sticky"
  | "stop"
  | "scrollUp"
  | "scrollDown"
  | "webToggle"
  | "reasoning"
  | "settings"
  | "models"
  | "follow"
  | "focusSidebar"
  | "toggleSidebar"
  | "editLast"
  | "prevChat"
  | "nextChat"
  | "cycleProfile"
  | "send"
  | "cancelEdit"
  | "sidebarUp"
  | "sidebarDown"
  | "openChat"
  | "pinChat"
  | "deleteChat"
  | "filterChats";

export type KeymapState = {
  composerEmpty: boolean;
  chatHasMessages: boolean;
  streaming: boolean;
  editing: boolean;
};

export function resolve(
  context: Context,
  event: KeyboardEvent,
  keybinds: Keybinds,
  state: KeymapState,
): Action | null {
  if (matchesAny(keybinds.palette, event)) return "palette";
  if (matchesAny(keybinds.sticky, event)) return "sticky";
  if (matchesAny(keybinds.sidebar, event)) return "toggleSidebar";
  if (matchesAny(keybinds.webToggle, event)) return "webToggle";
  if (matchesAny(keybinds.reasoning, event)) return "reasoning";
  if (matchesAny(keybinds.settings, event)) return "settings";
  if (matchesAny(keybinds.models, event)) return "models";
  if (matchesAny(keybinds.follow, event)) return "follow";
  if (matchesAny(keybinds.focusSidebar, event)) return "focusSidebar";
  if (state.streaming && matchesAny(keybinds.stop, event)) return "stop";
  if (matchesAny(keybinds.cycleProfile, event)) return "cycleProfile";

  if (context === "composer" || context === "sidebar") {
    if (matchesAny(keybinds.scrollUp, event)) return "scrollUp";
    if (matchesAny(keybinds.scrollDown, event)) return "scrollDown";
  }

  switch (context) {
    case "composer":
      return resolveComposer(eventToBinding(event), event, keybinds, state);
    case "sidebar":
      return resolveSidebar(eventToBinding(event), event, keybinds);
    case "palette":
      return resolvePalette(eventToBinding(event), event, keybinds);
    case "filter":
      return resolveFilter(eventToBinding(event), event, keybinds);
    case "confirm":
      return resolveConfirm(eventToBinding(event));
    default:
      return null;
  }
}

function resolveComposer(
  key: string,
  event: KeyboardEvent,
  keybinds: Keybinds,
  state: KeymapState,
): Action | null {
  if (key === "Esc") return state.editing ? "cancelEdit" : "hide";
  if (state.composerEmpty && key === "Left") return "toggleSidebar";

  if (state.composerEmpty && state.chatHasMessages && key === "Up") return "editLast";
  if (matchesAny(keybinds.prevChat, event)) return "prevChat";
  if (state.composerEmpty && !state.chatHasMessages && key === "Up") return "prevChat";

  if (matchesAny(keybinds.nextChat, event)) return "nextChat";
  if (state.composerEmpty && key === "Down") return "nextChat";

  if (key === "Enter" && !event.shiftKey) return "send";

  return null;
}

function resolveSidebar(key: string, event: KeyboardEvent, keybinds: Keybinds): Action | null {
  if (key === "Esc") return "hide";
  if (key === "Left") return "toggleSidebar";
  if (key === "J" || key === "Down" || matchesAny(keybinds.nextChat, event)) return "sidebarDown";
  if (key === "K" || key === "Up" || matchesAny(keybinds.prevChat, event)) return "sidebarUp";
  if (key === "Enter" || key === "L" || key === "Right" || key === "Ctrl+L") return "openChat";
  if (key === "P") return "pinChat";
  if (key === "D") return "deleteChat";
  if (key === "/") return "filterChats";
  return null;
}

function resolvePalette(key: string, event: KeyboardEvent, keybinds: Keybinds): Action | null {
  if (key === "Esc") return "paletteClose";
  if (key === "Down" || matchesAny(keybinds.nextChat, event)) return "paletteDown";
  if (key === "Up" || matchesAny(keybinds.prevChat, event)) return "paletteUp";
  if (key === "Enter") return "paletteRun";
  return null;
}

function resolveFilter(key: string, event: KeyboardEvent, keybinds: Keybinds): Action | null {
  if (key === "Esc") return "filterClose";
  if (key === "Enter") return "openChat";
  if (key === "Down" || matchesAny(keybinds.nextChat, event)) return "sidebarDown";
  if (key === "Up" || matchesAny(keybinds.prevChat, event)) return "sidebarUp";
  return null;
}

function resolveConfirm(key: string): Action | null {
  if (key === "Y" || key === "Enter") return "confirmDelete";
  if (key === "N" || key === "Esc") return "cancelDelete";
  return null;
}
