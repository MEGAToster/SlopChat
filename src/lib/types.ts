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
  | { type: "usage"; tokensIn: number; tokensOut: number; cost: number | null }
  | { type: "done" }
  | { type: "error"; message: string };

export type ChatMessage = {
  role: string;
  content: string;
};

export type SendRequest = {
  chatId: string;
  model: string;
  system?: string;
  temperature?: number;
  messages: ChatMessage[];
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
};
