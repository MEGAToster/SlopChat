import type { ThemeInfo } from "../../types";
import defaultDark from "./default-dark.json";
import defaultLight from "./default-light.json";
import nord from "./nord.json";

export const BUILTIN_THEMES: ThemeInfo[] = [
  { name: "default-dark", tokens: defaultDark },
  { name: "default-light", tokens: defaultLight },
  { name: "nord", tokens: nord },
];
