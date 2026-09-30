import { invoke, type Channel } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Attachment,
  Chat,
  ChatView,
  NewMessage,
  ProviderModel,
  SearchUsage,
  SendRequest,
  Settings,
  StoredMessage,
  StreamEvent,
  ThemeInfo,
} from "./types";

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

export function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export function getConfigError(): Promise<string | null> {
  return invoke("config_error");
}

export function resizePanel(height: number): Promise<void> {
  return invoke("resize_panel", { height });
}

export function hasApiKey(provider: string): Promise<boolean> {
  return invoke("has_api_key", { provider });
}

export function storeApiKey(provider: string, key: string): Promise<void> {
  return invoke("store_api_key", { provider, key });
}

export function deleteApiKey(provider: string): Promise<void> {
  return invoke("delete_api_key", { provider });
}

export function send(request: SendRequest, channel: Channel<StreamEvent>): Promise<void> {
  return invoke("send", { request, channel });
}

export function stop(chatId: string): Promise<void> {
  return invoke("stop", { chatId });
}

export function setPanelExpanded(expanded: boolean): Promise<void> {
  return invoke("set_panel_expanded", { expanded });
}

export function isPanelExpanded(): Promise<boolean> {
  return invoke("is_panel_expanded");
}

export function initialChat(): Promise<ChatView | null> {
  return invoke("initial_chat");
}

export function listChats(): Promise<Chat[]> {
  return invoke("list_chats");
}

export function createChat(profile: string): Promise<Chat> {
  return invoke("create_chat", { profile });
}

export function getChat(chatId: string): Promise<ChatView | null> {
  return invoke("get_chat", { chatId });
}

export function deleteChat(chatId: string): Promise<void> {
  return invoke("delete_chat", { chatId });
}

export function toggleChatPin(chatId: string): Promise<Chat> {
  return invoke("toggle_chat_pin", { chatId });
}

export function setChatProfile(chatId: string, profile: string): Promise<void> {
  return invoke("set_chat_profile", { chatId, profile });
}

export function appendMessage(message: NewMessage): Promise<StoredMessage> {
  return invoke("append_message", { message });
}

export function rewindTo(chatId: string, messageId: string, content: string): Promise<void> {
  return invoke("rewind_to", { chatId, messageId, content });
}

export function searchChats(query: string): Promise<Chat[]> {
  return invoke("search_chats", { query });
}

export function listModels(): Promise<ProviderModel[]> {
  return invoke("list_models");
}

export function openSettingsFile(): Promise<void> {
  return invoke("open_settings_file");
}

export function reloadConfig(): Promise<void> {
  return invoke("reload_config");
}

export function addModel(model: string): Promise<void> {
  return invoke("add_model", { model });
}

export function removeModel(model: string): Promise<void> {
  return invoke("remove_model", { model });
}

export function setTheme(theme: string): Promise<void> {
  return invoke("set_theme", { theme });
}

export function setStyle(style: string): Promise<void> {
  return invoke("set_style", { style });
}

export function loadUserThemes(): Promise<ThemeInfo[]> {
  return invoke("load_user_themes");
}

export function openUrl(url: string): Promise<void> {
  return invoke("open_url", { url });
}

export function setWindowStyle(
  scheme: string,
  vibrancy: string,
  opacity: number,
): Promise<void> {
  return invoke("set_window_style", { scheme, vibrancy, opacity });
}

export function setSuppressHide(value: boolean): Promise<void> {
  return invoke("set_suppress_hide", { value });
}

export function searchUsage(): Promise<SearchUsage> {
  return invoke("search_usage");
}

export function saveAttachment(name: string, mime: string, data: string): Promise<Attachment> {
  return invoke("save_attachment", { name, mime, data });
}

export function readAttachment(path: string): Promise<string> {
  return invoke("read_attachment", { path });
}

export function onPanelShown(handler: () => void): Promise<UnlistenFn> {
  return listen("panel://shown", handler);
}

export function onStickyChanged(handler: (sticky: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("panel://sticky", (event) => handler(event.payload));
}

export function onConfigChanged(handler: (settings: Settings) => void): Promise<UnlistenFn> {
  return listen<Settings>("config://changed", (event) => handler(event.payload));
}

export function onConfigError(handler: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>("config://error", (event) => handler(event.payload));
}
