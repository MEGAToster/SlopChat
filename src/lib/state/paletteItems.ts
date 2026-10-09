import { fuzzyMatch } from "../fuzzy";
import type { Chat, ProviderModel, Settings, ThemeInfo } from "../types";
import type { PaletteGroup, PaletteItem } from "./paletteTypes";

function narrow(items: PaletteItem[], query: string): PaletteItem[] {
  if (!query) return items;
  return items
    .map((item) => ({ item, match: fuzzyMatch(query, item.label) }))
    .filter((entry) => entry.match !== null)
    .sort((a, b) => b.match!.score - a.match!.score)
    .map((entry) => entry.item);
}

// A query matching the category title keeps every entry, so "models" lists
// all models rather than only those whose own label contains the query.
function category(title: string, items: PaletteItem[], query: string): PaletteItem[] {
  if (query && fuzzyMatch(query, title)) return items;
  return narrow(items, query);
}

export function rootGroups(input: {
  settings: Settings;
  chats: Chat[];
  themes: ThemeInfo[];
  query: string;
}): PaletteGroup[] {
  const query = input.query.trim();
  return [
    { title: "Chats", items: chatItems(input.chats) },
    { title: "Actions", items: category("Actions", actionItems(input.settings), query) },
    { title: "Models", items: category("Models", modelItems(input.settings), query) },
    { title: "Profiles", items: category("Profiles", profileItems(input.settings), query) },
    { title: "Themes", items: category("Themes", themeItems(input.themes), query) },
  ].filter((group) => group.items.length > 0);
}

export function addGroups(models: ProviderModel[], query: string): PaletteGroup[] {
  const items = models
    .slice()
    .sort((a, b) => a.provider.localeCompare(b.provider) || a.id.localeCompare(b.id))
    .map((model) => ({
      kind: "provider" as const,
      id: `${model.provider}/${model.id}`,
      label: model.name || model.id,
      provider: model.provider,
    }));
  return [
    { title: "Provider models", items: category("Provider models", items, query.trim()) },
  ].filter((group) => group.items.length > 0);
}

function actionItems(settings: Settings): PaletteItem[] {
  const keys = settings.keybinds;
  const actions: { id: string; label: string; binding?: string }[] = [
    { id: "new-chat", label: "New chat" },
    { id: "toggle-expanded", label: "Toggle expanded", binding: keys.sidebar?.[0] },
    { id: "toggle-sticky", label: "Toggle sticky", binding: keys.sticky?.[0] },
    { id: "switch-style", label: "Switch style" },
    { id: "cycle-reading-width", label: `Reading width: ${settings.readingWidth}` },
    { id: "search-usage", label: "Search usage" },
    { id: "pin-chat", label: "Pin / unpin chat" },
    { id: "rename-chat", label: "Rename chat" },
    { id: "add-models", label: "Add models…" },
    { id: "settings-panel", label: "Settings" },
    { id: "config-file", label: "Edit config file" },
    { id: "models-file", label: "Edit models file" },
    { id: "agent-log", label: "Open agent log" },
    { id: "reload", label: "Reload config" },
  ];
  return actions.map((action) => ({ kind: "action", ...action }));
}

function chatItems(chats: Chat[]): PaletteItem[] {
  return chats.map((chat) => ({ kind: "chat", id: chat.id, label: chat.title || "New chat" }));
}

function modelItems(settings: Settings): PaletteItem[] {
  return settings.models.map((entry) => ({ kind: "model", id: entry.model, label: entry.model }));
}

function profileItems(settings: Settings): PaletteItem[] {
  return settings.profiles.map((profile) => ({
    kind: "profile",
    id: profile.name,
    label: profile.name,
  }));
}

function themeItems(themes: ThemeInfo[]): PaletteItem[] {
  return themes.map((theme) => ({ kind: "theme", id: theme.name, label: theme.name }));
}
