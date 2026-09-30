export type Keybinds = {
  toggle: string[];
  palette: string[];
  sidebar: string[];
  prevChat: string[];
  nextChat: string[];
  cycleProfile: string[];
  sticky: string[];
  stop: string[];
  scrollUp: string[];
  scrollDown: string[];
};

export type Provider = {
  baseUrl: string;
  apiKey?: string;
};

export type Profile = {
  name: string;
  model: string;
  system?: string;
  temperature?: number;
  useContext: boolean;
};

export type Startup = "last" | "new";

export type WindowConfig = {
  compactWidth: number;
  expandedScreenRatio: number;
};

export type Settings = {
  keybinds: Keybinds;
  providers: Record<string, Provider>;
  profiles: Profile[];
  models: string[];
  startup: Startup;
  resumeWithinMinutes: number;
  theme: string;
  window: WindowConfig;
};

export type Usage = {
  tokensIn: number;
  tokensOut: number;
  cost: number | null;
};

export type StreamEvent =
  | { type: "delta"; content: string }
  | { type: "reasoning" }
  | { type: "usage"; tokensIn: number; tokensOut: number; cost: number | null }
  | { type: "done" }
  | { type: "error"; message: string };

export type ContentPart =
  | { type: "text"; text: string }
  | { type: "image_url"; image_url: { url: string } };

export type RequestMessage = {
  role: string;
  content: string | ContentPart[];
};

export type SendRequest = {
  chatId: string;
  model: string;
  system?: string;
  temperature?: number;
  useContext?: boolean;
  messages: RequestMessage[];
};

export type Attachment = {
  id: string;
  path: string;
  mime: string | null;
};

export type NewAttachment = {
  id: string;
  path: string;
  mime?: string;
};

export type Theme = {
  scheme?: "dark" | "light";
  extends?: string;
  vibrancy?: "hud" | "popover" | "sidebar" | "none";
  opacity?: number;
  colors?: Record<string, string>;
  syntax?: Record<string, string>;
  font?: { ui?: string; mono?: string; size?: number };
};

export type ThemeInfo = {
  name: string;
  theme: Theme;
};

export type Chat = {
  id: string;
  title: string;
  profile: string;
  pinned: boolean;
  createdAt: number;
  updatedAt: number;
};

export type StoredMessage = {
  id: string;
  chatId: string;
  role: string;
  content: string;
  model: string | null;
  tokensIn: number | null;
  tokensOut: number | null;
  cost: number | null;
  createdAt: number;
  attachments: Attachment[];
};

export type ChatView = {
  chat: Chat;
  messages: StoredMessage[];
};

export type NewMessage = {
  chatId: string;
  role: string;
  content: string;
  model?: string;
  tokensIn?: number;
  tokensOut?: number;
  cost?: number;
  attachments?: NewAttachment[];
};

export type ProviderModel = {
  provider: string;
  id: string;
  name?: string | null;
};
