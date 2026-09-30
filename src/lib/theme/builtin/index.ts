import type { Theme, ThemeInfo } from "../../types";
import defaultDark from "./default-dark.json";
import defaultLight from "./default-light.json";
import nord from "./nord.json";

export const BUILTIN_THEMES: ThemeInfo[] = [
  { name: "default-dark", theme: defaultDark as Theme },
  { name: "default-light", theme: defaultLight as Theme },
  { name: "nord", theme: nord as Theme },
];
