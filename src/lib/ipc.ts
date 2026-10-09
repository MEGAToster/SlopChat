import { invoke, type Channel } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Advanced,
  AntigravityStatus,
  ApiKeyStatus,
  Attachment,
  Chat,
  ChatView,
  ClipboardContent,
  FontSettings,
  ModelConfig,
  NewMessage,
  PlatformInfo,
  Profile,
  ProviderModel,
  ReadingWidth,
  ReasoningInfo,
  SavedFile,
  SearchProviderUsage,
  SendRequest,
  Settings,
  Startup,
  StoredMessage,
  StreamEvent,
  ThemeInfo,
  WindowConfig,
} from "./types";

export function hidePanel(): Promise<void> {
  return invoke("hide_panel");
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

export function resizeOverlay(height: number): Promise<void> {
  return invoke("resize_overlay", { height });
}

export function hasApiKey(provider: string): Promise<boolean> {
  return invoke("has_api_key", { provider });
}

export function storeApiKey(provider: string, key: string): Promise<void> {
  return invoke("store_api_key", { provider, key });
}

export function verifyApiKey(provider: string, key: string): Promise<ApiKeyStatus> {
  return invoke("verify_api_key", { provider, key });
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

export function renameChat(chatId: string, title: string): Promise<void> {
  return invoke("rename_chat", { chatId, title });
}

export function setChatModel(chatId: string, model: string | null): Promise<void> {
  return invoke("set_chat_model", { chatId, model });
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

export function openModelsFile(): Promise<void> {
  return invoke("open_models_file");
}

export function reloadConfig(): Promise<void> {
  return invoke("reload_config");
}

export function saveModel(model: ModelConfig, previous?: string): Promise<void> {
  return invoke("save_model", { model, previous });
}

export function removeModel(model: string): Promise<void> {
  return invoke("remove_model", { model });
}

export function saveProfile(profile: Profile, previous?: string): Promise<void> {
  return invoke("save_profile", { profile, previous });
}

export function removeProfile(name: string): Promise<void> {
  return invoke("remove_profile", { name });
}

export function modelReasoning(model: string): Promise<ReasoningInfo | null> {
  return invoke("model_reasoning", { model });
}

export function setTheme(theme: string): Promise<void> {
  return invoke("set_theme", { theme });
}

export function setStyle(style: string): Promise<void> {
  return invoke("set_style", { style });
}

export function setReadingWidth(width: ReadingWidth): Promise<void> {
  return invoke("set_reading_width", { width });
}

export function saveAdvanced(advanced: Advanced): Promise<void> {
  return invoke("save_advanced", { advanced });
}

export function saveWindow(window: WindowConfig): Promise<void> {
  return invoke("save_window", { window });
}

export function saveFont(font: FontSettings): Promise<void> {
  return invoke("save_font", { font });
}

export function setStartup(startup: Startup): Promise<void> {
  return invoke("set_startup", { startup });
}

export function setResumeWithin(minutes: number): Promise<void> {
  return invoke("set_resume_within", { minutes });
}

export function setPasteAsFile(chars: number): Promise<void> {
  return invoke("set_paste_as_file", { chars });
}

export function setReaderFallback(enabled: boolean): Promise<void> {
  return invoke("set_reader_fallback", { enabled });
}

export function setSearxngUrl(url: string): Promise<void> {
  return invoke("set_searxng_url", { url });
}

export function setSearxngInterval(ms: number): Promise<void> {
  return invoke("set_searxng_interval", { ms });
}

export function probeSearxng(): Promise<string | null> {
  return invoke("probe_searxng");
}

export function setAgentLog(enabled: boolean): Promise<void> {
  return invoke("set_agent_log", { enabled });
}

export function openAgentLog(): Promise<void> {
  return invoke("open_agent_log");
}

export function loadUserThemes(): Promise<ThemeInfo[]> {
  return invoke("load_user_themes");
}

export function openUrl(url: string): Promise<void> {
  return invoke("open_url", { url });
}

export function setWindowStyle(scheme: string, vibrancy: string): Promise<void> {
  return invoke("set_window_style", { scheme, vibrancy });
}

export function setSuppressHide(value: boolean): Promise<void> {
  return invoke("set_suppress_hide", { value });
}

export function readClipboard(limit: number): Promise<ClipboardContent> {
  return invoke("read_clipboard", { limit });
}

export function importPaths(paths: string[]): Promise<SavedFile[]> {
  return invoke("import_paths", { paths });
}

export function platformInfo(): Promise<PlatformInfo> {
  return invoke("platform_info");
}

export function searchUsage(): Promise<SearchProviderUsage[]> {
  return invoke("search_usage");
}

export function onSearchUnavailable(handler: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>("search://unavailable", (event) => handler(event.payload));
}

export function onSearchPaid(handler: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>("search://paid", (event) => handler(event.payload));
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

export function antigravityStatus(): Promise<AntigravityStatus> {
  return invoke("antigravity_status");
}
