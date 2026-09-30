import { fuzzyMatch } from "../fuzzy";
import type { Chat, ProviderModel, Settings, ThemeInfo } from "../types";
import type { PaletteGroup, PaletteItem } from "./paletteTypes";

export function narrow(items: PaletteItem[], query: string): PaletteItem[] {
  if (!query) return items;
  return items
    .map((item) => ({ item, match: fuzzyMatch(query, item.label) }))
    .filter((entry) => entry.match !== null)
    .sort((a, b) => b.match!.score - a.match!.score)
    .map((entry) => entry.item);
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
    { title: "Models", items: narrow(modelItems(input.settings), query) },
    { title: "Profiles", items: narrow(profileItems(input.settings), query) },
    { title: "Themes", items: narrow(themeItems(input.themes), query) },
    { title: "Actions", items: narrow(actionItems(input.settings), query) },
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
  return [{ title: "Provider models", items: narrow(items, query.trim()) }];
}

function actionItems(settings: Settings): PaletteItem[] {
  const keys = settings.keybinds;
  const actions: { id: string; label: string; binding?: string }[] = [
    { id: "new-chat", label: "New chat", binding: keys.nextChat?.[0] },
    { id: "toggle-expanded", label: "Toggle expanded", binding: keys.sidebar?.[0] },
    { id: "toggle-sticky", label: "Toggle sticky", binding: keys.sticky?.[0] },
    { id: "switch-style", label: "Switch style" },
    { id: "search-usage", label: "Search usage" },
    { id: "pin-chat", label: "Pin / unpin chat" },
    { id: "add-models", label: "Add models…" },
    { id: "settings-panel", label: "Settings" },
    { id: "config-file", label: "Edit config file" },
    { id: "reload", label: "Reload config" },
  ];
  return actions.map((action) => ({ kind: "action", ...action }));
}

function chatItems(chats: Chat[]): PaletteItem[] {
  return chats.map((chat) => ({ kind: "chat", id: chat.id, label: chat.title || "New chat" }));
}

function modelItems(settings: Settings): PaletteItem[] {
  const seen = new Set<string>();
  const items: PaletteItem[] = [];
  for (const model of [...settings.models, ...settings.profiles.map((p) => p.model)]) {
    if (seen.has(model)) continue;
    seen.add(model);
    items.push({ kind: "model", id: model, label: model });
  }
  return items;
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
