import type { Keybinds } from "../types";
import {
  resolve,
  resolveSettings,
  type Action,
  type Context,
  type KeymapState,
  type SettingsFocus,
} from "./dispatch";

type KeyHandlerActions = {
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
  settingsMove: (delta: number) => void;
  settingsActivate: () => void;
  settingsExpand: () => void;
  settingsCollapse: () => void;
};

type KeyHandlerDeps = {
  keybinds: () => Keybinds;
  state: () => KeymapState;
  context: () => Context;
  paletteOpen: () => boolean;
  paletteMode: () => string;
  settingsOpen: () => boolean;
  settingsBack: () => boolean;
  settingsFocus: () => SettingsFocus;
  blocked: () => boolean;
  filterActive: () => boolean;
  confirmActive: () => boolean;
  actions: KeyHandlerActions;
};

export function createKeyHandler(deps: KeyHandlerDeps): (event: KeyboardEvent) => void {
  const { actions } = deps;

  return (event: KeyboardEvent) => {
    if (deps.settingsOpen()) {
      const action = resolveSettings(event, deps.settingsFocus());
      if (!action) return;
      event.preventDefault();
      switch (action) {
        case "settingsBack":
          // An open editor pops back to the settings list; otherwise close.
          if (!deps.settingsBack()) actions.closeSettings();
          break;
        case "settingsNext":
          actions.settingsMove(1);
          break;
        case "settingsPrev":
          actions.settingsMove(-1);
          break;
        case "settingsActivate":
          actions.settingsActivate();
          break;
        case "settingsExpand":
          actions.settingsExpand();
          break;
        case "settingsCollapse":
          actions.settingsCollapse();
          break;
        default:
          break;
      }
      return;
    }

    if (deps.blocked()) {
      if (event.key === "Escape") {
        event.preventDefault();
        actions.escChain();
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
      } else if (action === "settings") {
        event.preventDefault();
        // Open first so closing the palette skips its composer-focus restore.
        actions.perform("settings");
        actions.paletteClose();
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

    if (event.code === "Tab" || event.keyCode === 9 || event.key === "Tab" || event.key === "ISO_Left_Tab") {
      event.preventDefault();
    }

    const action = resolve(deps.context(), event, deps.keybinds(), deps.state());
    if (!action) return;

    event.preventDefault();
    if (action === "hide") actions.escChain();
    else actions.perform(action);
  };
}
