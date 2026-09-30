import type { Theme, ThemeInfo } from "../../types";
import catppuccinFrappe from "./catppuccin-frappe.json";
import catppuccinLatte from "./catppuccin-latte.json";
import catppuccinMacchiato from "./catppuccin-macchiato.json";
import catppuccinMocha from "./catppuccin-mocha.json";
import defaultDark from "./default-dark.json";
import defaultLight from "./default-light.json";
import gruvboxDark from "./gruvbox-dark.json";
import gruvboxLight from "./gruvbox-light.json";
import nord from "./nord.json";
import rosePine from "./rose-pine.json";
import tokyoNight from "./tokyo-night.json";

export const BUILTIN_THEMES: ThemeInfo[] = [
  { name: "default-dark", theme: defaultDark as Theme },
  { name: "default-light", theme: defaultLight as Theme },
  { name: "catppuccin-mocha", theme: catppuccinMocha as Theme },
  { name: "catppuccin-macchiato", theme: catppuccinMacchiato as Theme },
  { name: "catppuccin-frappe", theme: catppuccinFrappe as Theme },
  { name: "catppuccin-latte", theme: catppuccinLatte as Theme },
  { name: "gruvbox-dark", theme: gruvboxDark as Theme },
  { name: "gruvbox-light", theme: gruvboxLight as Theme },
  { name: "nord", theme: nord as Theme },
  { name: "tokyo-night", theme: tokyoNight as Theme },
  { name: "rose-pine", theme: rosePine as Theme },
];
