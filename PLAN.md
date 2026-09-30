# SlopChat — Implementation Guide (for an LLM coding agent)

You are implementing **SlopChat**, a keyboard-first, lightweight chat app.
- It lives on the macOS Spotlight/Raycast panel layer, so tiling WMs and Space rules ignore it.
- It opens as a quick popup and expands into a full chat client.
- macOS first; Linux must stay possible.

Read this whole document before writing code. Work **milestone by milestone**. Do not start a milestone until the previous one meets its acceptance criteria.

## 0. Ground rules (non-negotiable)
1. **Lightweight.** Every dependency must earn its place. Prefer the standard library and small crates/packages. No UI component libraries, no CSS frameworks, no state-management libraries.
   - Targets: release bundle under ~15 MB, idle RAM under ~100 MB in total.
2. **Small files.** One purpose per file. Aim for under 200 lines and split before 300.
3. **Comments:**
   - Minimal and concise, and they explain *why*, not *what*.
   - Never write history ("changed from X", "previously", "now uses", "fixed"), TODO chatter, or commented-out code.
4. **Thin layers.**
   - Tauri commands only validate and delegate.
   - Platform-specific code lives only in `panel/` and `platform/` modules.
5. **No secrets on disk.** API keys live in the OS keychain only.
6. **Keep the codebase small.** Don't build abstractions for things that exist once. No speculative features beyond this document.
7. At scaffold time, create `CLAUDE.md` at the repo root containing rules 1–6 verbatim, plus the commands for dev, build and test.

## 1. Stack
- **Tauri v2**, Rust backend, **Svelte 5 + TypeScript + Vite** frontend (runes, no SvelteKit).
- **Rust crates:**
  - `tauri-plugin-nspanel` (macOS panel)
  - `tauri-plugin-global-shortcut`
  - `tauri-plugin-single-instance`
  - `reqwest` (rustls, stream)
  - `serde`/`serde_json`
  - `rusqlite` (`bundled`, includes FTS5)
  - `keyring`
  - `notify` (config hot reload)
  - `tokio`
  - Check each plugin's README for its current Tauri v2 API before using it. APIs change.
- **npm packages:** `marked`, `dompurify`, `highlight.js` (core + selected languages), `katex` (lazy-loaded). Nothing else without a strong reason.

## 2. Architecture decisions (already made — do not revisit)
- **Overlay only, no normal app window.**
  - One `NSPanel` with two modes:
    - **compact**: ~680 px wide; composer plus the current thread, growing up to ~480 px tall.
    - **expanded**: ~70% of the visible screen, with the sidebar. Size is remembered per mode.
  - **Sticky** mode disables hide-on-blur.
- **macOS panel config:**
  - Style mask includes `NonactivatingPanel`.
  - The panel can become key, so keystrokes arrive without activating the app.
  - Collection behavior: `canJoinAllSpaces | fullScreenAuxiliary | transient`.
  - Floating level.
  - Activation policy `Accessory` (no Dock or ⌘Tab entry).
  - A tray icon offers settings and quit.
- **Hide rules:**
  - Hide on resign-key unless sticky.
  - Suppress auto-hide while a native file dialog is open.
  - Hide, never destroy, the webview.
  - On show, focus the composer.
- **Providers:** OpenRouter, Ollama and Gemini API all use the **OpenAI chat-completions format**. Build one streaming client; each provider only defines base URL, auth, model listing and usage/cost extraction.
  - OpenRouter: `https://openrouter.ai/api/v1`, Bearer key.
    - Send `"usage": {"include": true}` and read `usage.cost` from the final chunk.
    - Ignore SSE comment lines starting with `:`.
    - Models come from `/models`, which includes pricing.
    - Model IDs may carry variant suffixes (`:floor`, `:nitro`, `:online`, `:free`, `:thinking`). Pass them through untouched.
  - Ollama: `http://localhost:11434/v1`, no auth.
    - Send `stream_options: {include_usage: true}`.
    - Models come from `http://localhost:11434/api/tags`. Cost is always 0.
  - Gemini: `https://generativelanguage.googleapis.com/v1beta/openai/`, Bearer API key.
  - No Gemini consumer-subscription support.
- **Model references** in config use the form `<provider>/<model-id>`, e.g. `openrouter/anthropic/claude-sonnet-5:floor`. Split on the first `/` only.
- **Streaming to the UI:** Tauri v2 `Channel` passed into the `send` command. Support cancellation through a `stop` command using an abort handle stored per chat.
- **Secrets:** `keyring`, service `SlopChat`, account = provider name. An `env:VAR` override is allowed in config (`"apiKey": "env:OPENROUTER_API_KEY"`).
- **Paths:** XDG-style on every OS, respecting `$XDG_CONFIG_HOME` and `$XDG_DATA_HOME`.
  - `~/.config/SlopChat/settings.json`: created with defaults if missing. Hot-reloaded; if the new file is invalid, keep the old config and show an error toast.
  - `~/.config/SlopChat/themes/*.json`: user themes, in addition to the bundled ones.
  - `~/.config/SlopChat/context.md`: personal context, prepended to the system prompt unless the profile sets `useContext: false`.
  - `~/.local/share/SlopChat/history/chats.db` and `~/.local/share/SlopChat/history/attachments/`.

## 3. Keyboard model
All bindings come from `settings.json → keybinds` (action → list of key strings like `"Ctrl+P"`, `"Alt+Space"`), merged over defaults defined in code. The global toggle is re-registered live when changed.

Rules:
- **Ctrl+hjkl always navigate**, even while typing.
  - They intentionally override the native macOS text bindings (Ctrl+H backspace, Ctrl+K kill-line). Call `preventDefault`.
- **Arrow keys navigate only when the composer is empty.** Otherwise they move the cursor.
- **The dispatcher is context-aware:** contexts are `composer`, `sidebar`, `palette`, `editing`. The topmost context wins.

| Context | Key(s) | Action |
|---|---|---|
| global | Alt+Space (configurable) | Toggle panel |
| composer | Ctrl+H, or ← when empty | Focus sidebar (auto-switches compact → expanded) |
| composer | ↑ when empty **and** the chat has messages | Edit last user message |
| composer | Ctrl+K, or ↑ when empty in a blank chat | Previous (older) chat |
| composer | Ctrl+J, or ↓ when empty | Newer chat; past the newest, a blank new chat |
| composer | Shift+Tab when empty | Cycle agent profile |
| composer | Enter / Shift+Enter | Send / newline |
| any | Ctrl+P | Command palette |
| any | Esc | Stop streaming → close palette → leave sidebar → collapse expanded → hide |
| sidebar | j/k, Ctrl+J/K, ↑/↓ | Move selection |
| sidebar | Enter, l, Ctrl+L, → | Open chat, focus composer |
| sidebar | p / d / `/` | Toggle pin / delete (confirm) / filter |
| palette | Ctrl+J/K, ↑/↓, Enter | Move / run |

**Edit last message:**
- Loads the last user message into the composer (context `editing`).
- On send, it replaces that message, deletes every message after it, and regenerates.
- Esc cancels.

**Command palette:** a single fuzzy list over:
- Actions: new chat, toggle expanded, toggle sticky, pin chat, switch theme, open settings file, reload config.
- Chats (FTS5 search over titles and content), models (from providers), profiles.

## 4. Features
- **Agent profiles**: defined in `settings.json → profiles`.
  - Fields: `name`, `model`, optional `system`, `temperature`, `useContext` (default true).
  - Shift+Tab cycles them. The active profile shows as a badge by the composer and is stored per chat.
  - Defaults: `default`, `cheap` (a `:floor` model), `complex`, `translate`.
- **Token & cost:**
  - Stored per assistant message (`tokens_in`, `tokens_out`, `cost`).
  - The status line shows the last message's figures and the chat total.
  - If a provider returns tokens but no cost, compute it from the `/models` pricing (cached); otherwise show tokens only.
- **Chat titles:** the first user message truncated to 60 chars. No extra LLM call.
- **Chat pinning:** at most one pinned chat. The pinned chat opens on summon.
- **Startup behavior:** `startup: "last" | "new"` plus `resumeWithinMinutes` (default 10).
  - Rule: pinned chat wins. Otherwise, with `"last"`, reopen the last chat if it was hidden less than `resumeWithinMinutes` ago, else open a new chat.
- **Theming:**
  - Every color is a CSS custom property on `:root`.
  - A theme JSON is a flat map of token → color. It includes code-highlight tokens, which map to highlight.js classes through CSS variables.
  - Bundled: `default-dark`, `default-light`, plus one or two popular palettes.
  - Theme can be switched from the palette and persists to settings.
- **Markdown:**
  - Pipeline: `marked` → `DOMPurify` → `{@html}`.
  - Streaming: split the text into top-level blocks, cache the rendered HTML of finished blocks, and re-render only the last block per token.
  - `highlight.js` with a curated set of languages (~15).
  - `katex` is loaded dynamically the first time math appears.
  - Code blocks get a copy button. Long threads use a virtualized list.
- **Attachments** (milestone 7):
  - Drag-drop and paste (images and text files) are sent as OpenAI-style content parts.
  - Files are copied into `attachments/`.
  - Guard the native file picker with auto-hide suppression.

### `settings.json` defaults (shape)
```json
{
  "keybinds": {
    "toggle": ["Alt+Space"], "palette": ["Ctrl+P"],
    "sidebar": ["Ctrl+H"], "prevChat": ["Ctrl+K"], "nextChat": ["Ctrl+J"],
    "cycleProfile": ["Shift+Tab"]
  },
  "providers": {
    "openrouter": { "baseUrl": "https://openrouter.ai/api/v1" },
    "ollama": { "baseUrl": "http://localhost:11434/v1" },
    "gemini": { "baseUrl": "https://generativelanguage.googleapis.com/v1beta/openai/" }
  },
  "profiles": [
    { "name": "default", "model": "openrouter/anthropic/claude-sonnet-5" },
    { "name": "cheap", "model": "openrouter/deepseek/deepseek-chat:floor" },
    { "name": "complex", "model": "openrouter/anthropic/claude-opus-5-5" },
    { "name": "translate", "model": "openrouter/google/gemini-2.5-flash", "system": "Translate the user's text. Detect the source language; translate to English unless another target is given.", "useContext": false }
  ],
  "startup": "last",
  "resumeWithinMinutes": 10,
  "theme": "default-dark",
  "window": { "compactWidth": 680, "expandedScreenRatio": 0.7 }
}
```

## 5. Data model (SQLite, migrations via `PRAGMA user_version`)
```sql
chats(id TEXT PK, title TEXT, profile TEXT, pinned INTEGER DEFAULT 0,
      created_at INTEGER, updated_at INTEGER)
messages(id TEXT PK, chat_id TEXT REFERENCES chats ON DELETE CASCADE,
         role TEXT, content TEXT, model TEXT,
         tokens_in INTEGER, tokens_out INTEGER, cost REAL, created_at INTEGER)
attachments(id TEXT PK, message_id TEXT REFERENCES messages ON DELETE CASCADE,
            path TEXT, mime TEXT)
messages_fts USING fts5(content, content='messages', content_rowid='rowid')  -- sync via triggers
```
Enable `PRAGMA foreign_keys = ON` and WAL mode.

## 6. File structure
```
CLAUDE.md
src-tauri/src/
  main.rs            app setup, tray, activation policy, plugin registration
  commands.rs        thin #[tauri::command] surface
  paths.rs           XDG dirs
  secrets.rs         keychain get/set/delete
  cli.rs             `slopchat toggle` argv handling via single-instance
  config/            mod.rs (types, defaults, load), watch.rs, keybinds.rs (global shortcut)
  panel/             mod.rs (show/hide/toggle/resize/sticky), macos.rs, linux.rs
  providers/         mod.rs (types, model-ref parsing), client.rs (streaming), sse.rs,
                     openrouter.rs, ollama.rs, gemini.rs
  store/             mod.rs (open, migrations), chats.rs, messages.rs, search.rs
src/
  main.ts, App.svelte
  lib/ipc.ts                 typed wrappers around invoke/Channel/events
  lib/keymap/                defaults.ts, parse.ts, dispatch.ts
  lib/markdown/              render.ts, stream.ts, highlight.ts, math.ts
  lib/state/                 chat.svelte.ts, ui.svelte.ts, settings.svelte.ts
  lib/theme/                 apply.ts, builtin/*.json
  components/                Composer, MessageList, Message, Sidebar, Palette,
                             ProfileBadge, StatusLine, Toast (.svelte)
```
Split any file that grows past ~200 lines along a natural seam.

## 7. Milestones & acceptance criteria
Complete each milestone before moving on.

1. **Scaffold + panel spike**
   - Tasks: Tauri v2 + Svelte 5 scaffold, `CLAUDE.md`, the NSPanel setup from §2, hardcoded Alt+Space toggle, hide on Esc/blur, focus composer on show.
   - Accept when:
     - The panel appears on the current Space, including over fullscreen apps.
     - The previously active app keeps its menu bar.
     - Typing works immediately.
     - There's no Dock icon.
     - The tiling WM does not tile the panel.
2. **Config + keymap**
   - Tasks: `paths.rs`, `config/` with defaults written on first run, hot reload, configurable global hotkey, frontend keymap with contexts.
   - Accept when:
     - Changing `toggle` in `settings.json` rebinds live.
     - An invalid JSON file shows a toast and keeps the old config.
3. **Chat MVP**
   - Tasks: key onboarding screen (shown when no key is stored) saving to the keychain, OpenRouter streaming via Channel, stop/cancel, markdown pipeline, profiles + Shift+Tab, status line with tokens and cost.
   - Accept when:
     - Replies stream smoothly with code blocks highlighted.
     - Esc stops generation.
     - Cost is shown.
     - The key is in the keychain and not on disk.
4. **History + expanded mode**
   - Tasks: SQLite store, compact/expanded modes, sidebar, prev/next/new chat nav, edit last message, pinning, startup rules, sticky mode.
   - Accept when every §3 row except the palette works without the mouse, and history survives a restart.
5. **Command palette**
   - Accept when fuzzy search over actions, chats (FTS), models and profiles works, and every action is runnable from it.
6. **Providers + themes + context**
   - Tasks: Ollama and Gemini, OpenRouter variant toggles in the model picker, theme loading (bundled + user), `context.md`.
   - Accept when:
     - A local Ollama model streams.
     - Switching themes applies instantly and persists.
     - `context.md` affects replies.
7. **Attachments**
   - Accept when a dropped or pasted image gets a vision-model reply, and the file picker doesn't hide the panel.
8. **Linux**
   - Tasks: `panel/linux.rs` (always-on-top, skip taskbar, undecorated), `slopchat toggle` for Wayland compositors, README section with WM float rules (Hyprland, sway, i3).
   - Accept when the app builds and toggles on Linux (X11 hotkey and Wayland via CLI).

## 8. Testing
Keep tests few and focused on logic, not UI.
- **Rust:** SSE parser (chunks split across reads, `:` comments, `[DONE]`), model-ref parsing, config defaults merge, keybind string parsing, store CRUD + FTS.
- **Vitest:** keymap dispatch (context and empty-composer rules), markdown block cache (only the last block re-renders).
- **Before finishing each milestone:** run `cargo test`, `cargo clippy -- -D warnings`, `npm run check`, `npm test`, then manually walk through that milestone's acceptance criteria in `cargo tauri dev`.

## 9. Pitfalls
- Non-activating panels still need `canBecomeKeyWindow = true`, or the webview won't receive keys.
- Register the global shortcut once; unregister before re-registering on config change.
- Debounce file-watch events (editors write files in several steps).
- Sanitize all model output before `{@html}`.
- Re-rendering the whole message per token will stutter. Use the block cache.
- OpenRouter can send the usage chunk after the last content chunk; don't close the stream on `finish_reason`, wait for `[DONE]`.
