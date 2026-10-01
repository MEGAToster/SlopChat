type PaletteKind = "action" | "chat" | "model" | "provider" | "profile" | "theme";

export type PaletteItem = {
  kind: PaletteKind;
  id: string;
  label: string;
  provider?: string;
  binding?: string;
};

export type PaletteGroup = {
  title: string;
  items: PaletteItem[];
};
