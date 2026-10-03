# SlopChat

DISCLAIMER: This project has been written by AI and only had a human guiding it and testing the product. This is made by slop to better prompt more slop.

A keyboard-first, lightweight chat client that lives on the macOS Spotlight/Raycast panel layer and expands into a full chat window. It also builds and runs on Linux (Hyprland/Wayland; see [Linux](#linux)).

## Features

- **Panel UI** — one hotkey summons a compact composer; it grows into an expanded window with a sidebar. Sticky mode disables hide-on-blur. No Dock icon.
- **Providers** — OpenRouter, Ollama (all via the OpenAI chat-completions format). API keys live in the OS keychain.
- **Profiles** — named model configurations cycled with Shift+Tab and stored per chat. Optional fallback model per profile.
- **Streaming answers** — markdown, syntax highlighting, KaTeX math, Mermaid diagrams, inline citations, and copy buttons; long threads stay smooth.
- **Web research** — `web_search` (Exa, Tavily, Brave, Firecrawl, Serper, tried in order as **fallbacks** — see [Web search billing](#web-search-billing)) and `web_fetch` (readable text with an SSRF guard and a Jina Reader fallback for blocked/JS-only pages). Only `web_search` uses provider APIs; `web_fetch` is always the free local fetch.
- **Reasoning** — cycle effort levels for models that expose them.
- **History** — SQLite + FTS5 search, pinning, edit-last-message, and configurable startup/resume behavior.
- **Attachments** — drop or paste images and text files.
- **Command palette** — fuzzy search over chats, actions, models, profiles, and themes.
- **Themes & styles** — bundled + user themes, minimal and TUI styles, font options.
- **Advanced limits** — cap searches, fetches, fetched size, and tool rounds per reply; response-inactivity and total-turn timeouts; reasoning and completion-token caps; mild frequency/presence penalties (skipped for reasoning models); reader fallback and agent trace log.

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

The script checks for Node and Rust, builds the app, and installs it (macOS: `/Applications/SlopChat.app`; Linux: `~/.local/bin/slopchat` plus a desktop entry). To build manually instead: `npm install && npm run tauri build`, then use the artifact under `src-tauri/target/release/bundle/`.

On first launch, paste an OpenRouter API key (stored in the keychain). Add other providers and keys under **Settings → Web search**.

### Web search billing

The web-search providers are **fallbacks**, not a budget you can rely on:

- SlopChat tries the configured providers **in order** and only moves to the next one when the current provider **rejects the request** (auth failure, quota exhausted, HTTP error). A provider that answers successfully is used even if it means paying.
- The per-provider "free allowance" values in Settings are a **best-effort estimate** based on the requests *SlopChat itself has made*. They are **not** a hard cap: SlopChat never queries the provider's real balance, cannot see usage from other apps or devices, and cannot know your account status.
- **If a provider is set to pay-as-you-go / overage on demand, exceeding its free tier will be charged to you.** SlopChat will keep using it (and you may be billed) until the provider itself starts rejecting requests — only then does it fall back.

To avoid surprise charges, disable pay-on-demand at the provider, or don't store that provider's key in SlopChat. `web_fetch` is always free (local + optional Jina reader) and never touches these providers.

### macOS keychain prompts

If the app is unsigned, macOS asks for keychain access on every chat send, Settings open, and rebuild — and each "Always Allow" asks for your password — because every build has a different code signature. Fix it once with a stable self-signed identity.

> **Creating the certificate is not enough — you must TRUST it.** An untrusted certificate shows up as `CSSMERR_TP_NOT_TRUSTED`, `codesign` refuses it (`no identity found`), and the app silently builds unsigned, so the prompts keep coming. Trusting the cert is the step everyone misses.

1. Keychain Access → Certificate Assistant → **Create a Certificate**.
   - Name: `SlopChat Local Code Signing`
   - Identity Type: Self Signed Root; Certificate Type: **Code Signing**
2. **Trust it (required).** Double-click the certificate in the **login** keychain → expand **Trust** → set *When using this certificate* to **Always Trust** → close the window and enter your password.
   - Verify it worked — this must list `"SlopChat Local Code Signing"` **without** `CSSMERR_TP_NOT_TRUSTED`:
     ```sh
     security find-identity -v -p codesigning
     ```
   - Or trust it from a terminal (one password prompt):
     ```sh
     security find-certificate -c "SlopChat Local Code Signing" -p > ~/slopchat-cert.pem
     security add-trusted-cert -r trustRoot -p codeSign \
       -k ~/Library/Keychains/login.keychain-db ~/slopchat-cert.pem
     ```
3. Re-run `./scripts/install.sh`. It aborts until the certificate is trusted, then prints `Signing as: SlopChat Local Code Signing` and installs to `/Applications`. (Builds sign with this identity by default via `tauri.conf.json`; override with `APPLE_SIGNING_IDENTITY` if your cert has another name.)
4. On the first run after switching from an older build, click **Always Allow** (not just Allow) once per item; that one click asks for your password.
5. Run the **installed** `/Applications/SlopChat.app`. `npm run tauri dev` and the raw build output are unsigned, so they always prompt.

### Linux

Global shortcuts only work on X11. On Wayland (Hyprland, sway, …), run a resident instance and toggle it from a compositor keybind:

```sh
slopchat          # start a hidden resident instance
slopchat toggle   # show/hide the running instance
slopchat show     # show it
slopchat hide     # hide it
slopchat quit     # stop the resident instance
```

Hyprland (`hyprland.conf`):

``` lua
hl.on("hyprland.start", function () {
    hl.exec_cmd("slopchat")
}

hl.bind("ALT + SHIFT + Space", hl.dsp.exec_cmd("slopchat show"))

hl.layer_rule({
    name = "SlopChat",
    match = { namespace = "slopchat" },
    blur = true,
    ignore_alpha = 0,
    no_anim = true,
})


```

The panel uses the `wlr-layer-shell` overlay layer on Wayland (Hyprland, sway, KDE Plasma, Mir) so the compositor does not tile it; GNOME-on-Wayland and X11 fall back to a normal window. See [docs/linux.md](docs/linux.md) for the layer-shell internals, the tray, the Wayland global-shortcut caveat and fallback rules. API-key storage is in [docs/secrets.md](docs/secrets.md).

Build dependencies (Fedora): `webkit2gtk4.1-devel`, `libsoup3-devel`, `librsvg2-devel`, `libayatana-appindicator-gtk3-devel`, `gtk-layer-shell-devel`, `patchelf`.

## Configuration

- `~/.config/SlopChat/settings.json` — keybinds, providers, theme, limits (hot-reloaded).
- `~/.config/SlopChat/models.json` — registered models and profiles (hot-reloaded; safe to git-sync).
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
