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
  webToggle: string[];
  reasoning: string[];
  settings: string[];
  models: string[];
  follow: string[];
  focusSidebar: string[];
};

export type Provider = {
  baseUrl: string;
  apiKey?: string;
};

export type ModelConfig = {
  model: string;
  system?: string;
  temperature?: number;
  useContext: boolean;
  tools?: string[];
  toolCalling?: boolean;
  webSearch?: WebSearchMode;
  reasoning?: string;
};

export type ProfileOverrides = {
  system?: string;
  temperature?: number;
  useContext?: boolean;
  tools?: string[];
  toolCalling?: boolean;
  webSearch?: WebSearchMode;
  reasoning?: string;
};

export type Profile = {
  name: string;
  model: string;
  fallback?: string;
  color?: string;
} & ProfileOverrides;

export type ReasoningInfo = {
  supportedEfforts: string[];
  defaultEffort?: string;
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
  models: ModelConfig[];
  startup: Startup;
  resumeWithinMinutes: number;
  theme: string;
  style: "auto" | ThemeStyles;
  font?: FontSettings;
  window: WindowConfig;
};

export type Usage = {
  tokensIn: number;
  tokensOut: number;
  cost: number | null;
  provider?: string | null;
};

export type Source = {
  n: number;
  title: string;
  url: string;
};

export type ToolStep = {
  id: string;
  name: string;
  args: Record<string, unknown>;
  status: "running" | "done" | "error";
  summary: string;
};

export type WebSearchMode = "auto" | "always" | "off";

export type StreamEvent =
  | { type: "delta"; content: string }
  | { type: "reasoning" }
  | { type: "toolCall"; id: string; name: string; args: Record<string, unknown> }
  | { type: "toolResult"; id: string; ok: boolean; summary: string; sources: Source[] }
  | { type: "usage"; tokensIn: number; tokensOut: number; cost: number | null; provider: string | null }
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
  tools?: string[];
  toolCalling?: boolean;
  webSearch?: WebSearchMode;
  webToggle?: boolean;
  reasoning?: string;
  fallback?: string;
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

export type ThemeStyles = "minimal" | "tui";

export type Theme = {
  scheme?: "dark" | "light";
  style?: ThemeStyles;
  extends?: string;
  vibrancy?: "hud" | "popover" | "sidebar" | "none";
  opacity?: number;
  colors?: Record<string, string>;
  syntax?: Record<string, string>;
  palette?: Record<string, string>;
  font?: { ui?: string; mono?: string; size?: number };
};

export type FontSettings = { ui?: string; mono?: string; size?: number };

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
  sources: Source[] | null;
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
  sources?: Source[];
  attachments?: NewAttachment[];
};

export type ProviderModel = {
  provider: string;
  id: string;
  name?: string | null;
};

export type SearchLimit = {
  monthly?: number;
  monthlyUsd?: number;
  total?: number;
};

export type SearchProviderUsage = {
  provider: string;
  requests: number;
  units: number;
  totalUnits: number;
  cost: number;
  limit?: SearchLimit;
  exhausted: boolean;
  hasKey: boolean;
};
