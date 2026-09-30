import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function showPanel(): Promise<void> {
  return invoke("show_panel");
}

export function hidePanel(): Promise<void> {
  return invoke("hide_panel");
}

export function togglePanel(): Promise<void> {
  return invoke("toggle_panel");
}

export function toggleSticky(): Promise<boolean> {
  return invoke("toggle_sticky");
}

export function isSticky(): Promise<boolean> {
  return invoke("is_sticky");
}

export function onPanelShown(handler: () => void): Promise<UnlistenFn> {
  return listen("panel://shown", handler);
}

export function onStickyChanged(handler: (sticky: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("panel://sticky", (event) => handler(event.payload));
}
