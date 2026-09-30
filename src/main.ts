import { mount } from "svelte";

import App from "./App.svelte";
import { applyTheme, resolveTheme } from "./lib/theme/apply";
import { BUILTIN_THEMES } from "./lib/theme/builtin";
import "./styles/tokens.css";
import "./styles/base.css";
import "./styles/markdown.css";
import "./styles/syntax.css";

const builtin = new Map(BUILTIN_THEMES.map((entry) => [entry.name, entry.theme]));
applyTheme(resolveTheme(builtin, "default-dark"));

const target = document.getElementById("app");
if (!target) throw new Error("missing #app mount point");

mount(App, { target });
