import type { Keybinds } from "../types";
import { eventToBinding, matchesAny } from "./parse";

export type Context = "composer" | "sidebar" | "palette" | "editing";

export type Action =
  | "hide"
  | "palette"
  | "sticky"
  | "focusSidebar"
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
  | "filterChats"
  | "paletteUp"
  | "paletteDown"
  | "paletteRun";

export type KeymapState = {
  composerEmpty: boolean;
  chatHasMessages: boolean;
};

export function resolve(
  context: Context,
  event: KeyboardEvent,
  keybinds: Keybinds,
  state: KeymapState,
): Action | null {
  const key = eventToBinding(event);

  if (context === "editing" && key === "Esc") return "cancelEdit";
  if (matchesAny(keybinds.palette, event)) return "palette";
  if (matchesAny(keybinds.sticky, event)) return "sticky";
  if (key === "Esc") return "hide";

  switch (context) {
    case "composer":
      return resolveComposer(key, event, keybinds, state);
    case "sidebar":
      return resolveSidebar(key, event, keybinds);
    case "palette":
      return resolvePalette(key, event, keybinds);
    case "editing":
      return key === "Enter" && !event.shiftKey ? "send" : null;
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
  if (matchesAny(keybinds.sidebar, event)) return "focusSidebar";
  if (state.composerEmpty && key === "Left") return "focusSidebar";

  if (state.composerEmpty && state.chatHasMessages && key === "Up") return "editLast";
  if (matchesAny(keybinds.prevChat, event)) return "prevChat";
  if (state.composerEmpty && !state.chatHasMessages && key === "Up") return "prevChat";

  if (matchesAny(keybinds.nextChat, event)) return "nextChat";
  if (state.composerEmpty && key === "Down") return "nextChat";

  if (state.composerEmpty && matchesAny(keybinds.cycleProfile, event)) return "cycleProfile";
  if (key === "Enter" && !event.shiftKey) return "send";

  return null;
}

function resolveSidebar(key: string, event: KeyboardEvent, keybinds: Keybinds): Action | null {
  if (key === "J" || key === "Down" || matchesAny(keybinds.nextChat, event)) return "sidebarDown";
  if (key === "K" || key === "Up" || matchesAny(keybinds.prevChat, event)) return "sidebarUp";
  if (key === "Enter" || key === "L" || key === "Right" || key === "Ctrl+L") return "openChat";
  if (key === "P") return "pinChat";
  if (key === "D") return "deleteChat";
  if (key === "/") return "filterChats";
  return null;
}

function resolvePalette(key: string, event: KeyboardEvent, keybinds: Keybinds): Action | null {
  if (key === "Down" || matchesAny(keybinds.nextChat, event)) return "paletteDown";
  if (key === "Up" || matchesAny(keybinds.prevChat, event)) return "paletteUp";
  if (key === "Enter") return "paletteRun";
  return null;
}
