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
  zoomIn: string[];
  zoomOut: string[];
  zoomReset: string[];
};

type Provider = {
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
  maxCompletionTokens?: number;
  routing?: Routing;
};

// OpenRouter provider-routing hints; other providers ignore them.
export type MaxPrice = {
  prompt?: number;
  completion?: number;
};

export type Routing = {
  order?: string[];
  only?: string[];
  ignore?: string[];
  allowFallbacks?: boolean;
  sort?: string;
  maxPrice?: MaxPrice;
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

export type Startup = "pinned" | "latest" | "new";

export type Advanced = {
  maxSearches: number;
  maxFetches: number;
  maxRounds: number;
  responseTimeoutSecs: number;
  maxTurnSecs: number;
  maxFetchedChars: number;
  reasoningMaxTokens: number;
  frequencyPenalty: number;
  presencePenalty: number;
  deep: DeepSettings;
};

export type DeepSettings = {
  maxRounds: number;
  maxSearches: number;
  maxFetches: number;
  maxFetchedChars: number;
  maxTurnSecs: number;
  subagentModel: string;
};

export type SearchSettings = {
  providers: string[];
  readerFallback: boolean;
  searxng: { baseUrl: string };
};

type DebugSettings = {
  agentLog: boolean;
};

export type WebOverride = "off" | "auto" | "deep";

export type ReadingWidth = "narrow" | "wide" | "full";

export type WindowConfig = {
  compactWidth: number;
  compactHeight: number;
  expandedScreenRatio: number;
};

export type Settings = {
  keybinds: Keybinds;
  providers: Record<string, Provider>;
  profiles: Profile[];
  models: ModelConfig[];
  startup: Startup;
  resumeWithinMinutes: number;
  pasteAsFileChars: number;
  theme: string;
  style: "auto" | ThemeStyles;
  readingWidth: ReadingWidth;
  font?: FontSettings;
  search: SearchSettings;
  advanced: Advanced;
  debug: DebugSettings;
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
  status: "running" | "done" | "error" | "skipped";
  summary: string;
  note?: string;
};

export type WebSearchMode = "auto" | "deep" | "off";

export type StreamEvent =
  | { type: "delta"; content: string; round: number }
  | { type: "reasoning" }
  | { type: "model"; model: string }
  | { type: "toolCall"; id: string; name: string; args: Record<string, unknown> }
  | {
      type: "toolResult";
      id: string;
      ok: boolean;
      skipped?: boolean;
      summary: string;
      sources: Source[];
    }
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
  reasoning?: string;
  fallback?: string;
  maxCompletionTokens?: number;
  messages: RequestMessage[];
};

export type Attachment = {
  id: string;
  path: string;
  mime: string | null;
};

// A file already written to the attachments directory by the backend.
export type SavedFile = {
  id: string;
  path: string;
  mime: string;
  name: string;
  size: number;
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
  font?: FontSettings;
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
  model: string | null;
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
  provider: string | null;
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
  provider?: string;
  sources?: Source[];
  attachments?: NewAttachment[];
};

export type ProviderModel = {
  provider: string;
  id: string;
  name?: string | null;
};

type SearchLimit = {
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

export type AntigravityStatus = {
  path: string | null;
  version: string | null;
  ok: boolean;
  note: string | null;
};

export type ApiKeyStatus = "ok" | "rejected" | "unreachable";

export type ClipboardContent = {
  text: string | null;
  image: { pngBase64: string } | null;
  file: SavedFile | null;
};

export type PlatformInfo = {
  linux: boolean;
  wayland: boolean;
};
