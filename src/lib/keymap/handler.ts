import type { Keybinds } from "../types";
import { resolve, type Action, type Context, type KeymapState } from "./dispatch";

export type KeyHandlerActions = {
  perform: (action: Action) => void;
  paletteMove: (delta: number) => void;
  paletteActivate: () => void;
  paletteBack: () => void;
  paletteClose: () => void;
  closeSettings: () => void;
  closeFilter: () => void;
  openSelected: () => void;
  confirmDelete: () => void;
  cancelDelete: () => void;
  escChain: () => void;
};

export type KeyHandlerDeps = {
  keybinds: () => Keybinds;
  state: () => KeymapState;
  context: () => Context;
  paletteOpen: () => boolean;
  paletteMode: () => string;
  settingsOpen: () => boolean;
  filterActive: () => boolean;
  confirmActive: () => boolean;
  actions: KeyHandlerActions;
};

export function createKeyHandler(deps: KeyHandlerDeps): (event: KeyboardEvent) => void {
  const { actions } = deps;

  return (event: KeyboardEvent) => {
    if (deps.settingsOpen()) {
      if (event.key === "Escape") {
        event.preventDefault();
        actions.closeSettings();
      }
      return;
    }

    if (deps.paletteOpen()) {
      const action = resolve("palette", event, deps.keybinds(), deps.state());
      if (action === "palette" || action === "paletteClose") {
        event.preventDefault();
        if (action === "paletteClose" && deps.paletteMode() === "add") actions.paletteBack();
        else actions.paletteClose();
      } else if (action === "paletteRun") {
        event.preventDefault();
        actions.paletteActivate();
      } else if (action === "paletteUp") {
        event.preventDefault();
        actions.paletteMove(-1);
      } else if (action === "paletteDown") {
        event.preventDefault();
        actions.paletteMove(1);
      }
      return;
    }

    if (deps.confirmActive()) {
      const action = resolve("confirm", event, deps.keybinds(), deps.state());
      if (action === "confirmDelete") {
        event.preventDefault();
        actions.confirmDelete();
      } else if (action === "cancelDelete") {
        event.preventDefault();
        actions.cancelDelete();
      }
      return;
    }

    if (deps.filterActive()) {
      const action = resolve("filter", event, deps.keybinds(), deps.state());
      if (action === "filterClose") {
        event.preventDefault();
        actions.closeFilter();
      } else if (action === "openChat") {
        event.preventDefault();
        actions.openSelected();
      } else if (action === "sidebarUp" || action === "sidebarDown") {
        event.preventDefault();
        actions.perform(action);
      }
      return;
    }

    if (event.key === "Tab") event.preventDefault();

    const action = resolve(deps.context(), event, deps.keybinds(), deps.state());
    if (!action) return;

    event.preventDefault();
    if (action === "hide") actions.escChain();
    else actions.perform(action);
  };
}
