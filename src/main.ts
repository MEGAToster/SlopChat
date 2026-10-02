import { mount } from "svelte";

import App from "./App.svelte";
import { applyTheme, resolveTheme } from "./lib/theme/apply";
import { BUILTIN_THEMES } from "./lib/theme/builtin";
import "./styles/tokens.css";
import "./styles/base.css";
import "./styles/editor.css";
import "./styles/markdown.css";
import "./styles/syntax.css";
import "./styles/tui.css";
import "./styles/tui-app.css";
import "./styles/tui-markdown.css";

const builtin = new Map(BUILTIN_THEMES.map((entry) => [entry.name, entry.theme]));
applyTheme(resolveTheme(builtin, "default-dark"));

const target = document.getElementById("app");
if (!target) throw new Error("missing #app mount point");

mount(App, { target });

// Once the panel is painted, drop the startup black so vibrancy/compositor
// effects can show through the translucent theme background.
document.documentElement.style.background = "transparent";
