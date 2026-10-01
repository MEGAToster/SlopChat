# SlopChat

A keyboard-first, lightweight chat client that lives on the macOS Spotlight/Raycast panel layer and expands into a full chat window. Linux support is planned (`LINUX_PLAN.md`).

## Features

- **Panel UI** — one hotkey summons a compact composer; it grows into an expanded window with a sidebar. Sticky mode disables hide-on-blur. No Dock icon.
- **Providers** — OpenRouter, Ollama, and Gemini (all via the OpenAI chat-completions format). API keys live in the OS keychain.
- **Profiles** — named model configurations cycled with Shift+Tab and stored per chat. Optional fallback model per rotation entry.
- **Streaming answers** — markdown, syntax highlighting, KaTeX math, Mermaid diagrams, inline citations, and copy buttons; long threads stay smooth.
- **Web research** — `web_search` (Exa, Serper, Tavily, Brave, tried in order with free-allowance budgets) and `web_fetch` (readable text with an SSRF guard and a Jina Reader fallback for blocked/JS-only pages).
- **Reasoning** — cycle effort levels for models that expose them.
- **History** — SQLite + FTS5 search, pinning, edit-last-message, and configurable startup/resume behavior.
- **Attachments** — drop or paste images and text files.
- **Command palette** — fuzzy search over chats, actions, models, profiles, and themes.
- **Themes & styles** — bundled + user themes, minimal and TUI styles, font options.
- **Advanced limits** — cap searches, fetches, and tool rounds per reply; toggle the reader fallback and the agent trace log.

## Default keybinds

| Key | Action |
| --- | --- |
| `Alt+Space` | Toggle the panel |
| `Ctrl+P` | Command palette |
| `Ctrl+H` | Toggle sidebar (composer: focus sidebar; sidebar: collapse) |
| `Ctrl+S` | Focus sidebar |
| `Ctrl+J` / `Ctrl+K` | Newer / older chat |
| `Shift+Tab` | Cycle profile (composer empty) |
| `Ctrl+Shift+P` | Toggle sticky |
| `Ctrl+Q` | Stop streaming |
| `Ctrl+U` / `Ctrl+D` | Scroll up / down |
| `Ctrl+W` | Cycle web search: on → off → auto |
| `Ctrl+R` | Cycle reasoning effort |
| `Ctrl+,` | Settings |
| `Ctrl+M` | Model picker |
| `Ctrl+L` | Follow output (jump to bottom / track) |
| `Enter` / `Shift+Enter` | Send / newline |
| `Esc` | Stop → close → leave sidebar → collapse → hide |

Sidebar: `j`/`k` or arrows move, `Enter`/`l`/`→` open, `p` pin, `d` delete, `/` filter.
Palette: arrows or `Ctrl+J`/`Ctrl+K` move, `Enter` run.

All bindings are configurable in `settings.json` under `keybinds`.

## Install

Build from source and install:

```sh
./scripts/install.sh
```

The script checks for Node and Rust, builds the app, and installs it (macOS: `/Applications/SlopChat.app`; Linux: the `.deb` or AppImage). To build manually instead: `npm install && npm run tauri build`, then use the artifact under `src-tauri/target/release/bundle/`.

On first launch, paste an OpenRouter API key (stored in the keychain). Add other providers and keys under **Settings → Web search**.

### macOS keychain prompts

If the app is unsigned, macOS asks for keychain access on every chat send, Settings open, and rebuild, because each build has a different code signature. Fix it once with a stable self-signed identity:

1. Keychain Access → Certificate Assistant → **Create a Certificate**.
   - Name: `SlopChat Local Code Signing`
   - Identity Type: Self Signed Root; Certificate Type: **Code Signing**
2. Re-run `./scripts/install.sh` (it detects the certificate), or build with `APPLE_SIGNING_IDENTITY="SlopChat Local Code Signing" npm run tauri build`.
3. On the first run after switching from an older build, click **Always Allow** (not just Allow) once per item.

Note: `npm run tauri dev` builds are unsigned, so they will prompt on each rebuild — use the installed app for daily use.

### Linux

Global shortcuts do not work on Wayland — bind a compositor keybind to `slopchat toggle` instead, and set the window float rules. See `LINUX_PLAN.md`.

## Configuration

- `~/.config/SlopChat/settings.json` — keybinds, providers, profiles, models, theme, limits (hot-reloaded).
- `~/.config/SlopChat/context.md` — personal context prepended to the system prompt.
- `~/.config/SlopChat/themes/*.json` — user themes.
- `~/.local/share/SlopChat/history/` — chats database and attachments.
- `~/.local/share/SlopChat/logs/agent.jsonl` — agent trace, when enabled in Advanced settings.

## Choosing a model

Small "flash"-class models are weak at multi-step tool use: they may narrate instead of searching, give up after one failed fetch, or glue sentences together. For research-heavy prompts, add a `research` profile pointing at a stronger tool-capable model and keep the cheap model as your default. Turn on **Advanced → Agent log** to see exactly what each round did.

## Development

```sh
npm install
npm run tauri dev     # run the app
npm test              # frontend tests
npm run check         # svelte-check
cd src-tauri && cargo test && cargo clippy -- -D warnings
```

## Privacy

API keys are stored only in the OS keychain (or an `env:VAR` override in config), never in plaintext on disk. Fetched pages are retrieved directly; the optional reader fallback sends the URL to `r.jina.ai`.

## License

AGPL-3.0-or-later. See `LICENSE`.
