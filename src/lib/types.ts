export type Keybinds = {
  toggle: string[];
  palette: string[];
  sidebar: string[];
  prevChat: string[];
  nextChat: string[];
  cycleProfile: string[];
  sticky: string[];
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
