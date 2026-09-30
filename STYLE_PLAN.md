# SlopChat — Polish pass + TUI style & community themes (for an LLM coding agent)

Follow CLAUDE.md: small files, minimal comments, no new dependencies, no raw colors in components (enforced by `src/lib/theme/no-raw-colors.test.ts`).

**Context:**
- The minimal Raycast-style design (DESIGN_PLAN.md Part 2) is implemented.
- Screenshots show layout issues to polish (Part A).
- The user also wants a second, vibrant **TUI style** inspired by r/unixporn and btop, lazygit, yazi and fzf (Part B), plus community palettes: Catppuccin, Gruvbox, Nord, Tokyo Night, Rosé Pine (Part C).
- Style and palette are **independent axes**: any theme can be shown in either style.

Order: Part A → Part B → Part C. Each part ends with its acceptance checks.

---

## Part A — Polish the minimal style

1. **Dead space in compact mode.**
   - Cause: `MIN_HEIGHT = 160` in `src-tauri/src/panel/mod.rs`, while the empty state is ~88 px.
   - Fix: lower it to the measured empty height, and set `.content` to `justify-content: flex-end` in compact so leftover space never sits under the footer.
2. **Hairlines run edge to edge.** Remove the padding from `.content` in `src/styles/base.css`. Header, MessageList, Composer and Footer each own their horizontal padding.
3. **One content column.** Add a shared `.column` rule (`max-width: 760px; margin-inline: auto; padding-inline: var(--sp-4)`) and use it inside MessageList, Composer and Footer, so all left edges align in expanded mode.
4. **Remove the redundant header.** The chat title appears in the header, the sidebar row and the first bubble. Delete `Header.svelte` and move the sticky indicator into the Footer's right side.
5. **Distinct icon for sticky.** Sticky and pinned-chat both use `pin`. Add a `lock` icon for sticky and show it in the footer as `⌃⇧P Sticky`, in `accent` when on.
6. **Sidebar:**
   - Stronger `sidebar` tint: dark `rgba(0,0,0,.25)`, light `rgba(0,0,0,.05)`.
   - Active-chat bar inside the rounded row.
   - Titles at 13 px in `text-secondary`, with `text` for active or selected rows.
   - A permanent "Search chats `/`" row at the top.
7. **Heading hierarchy** (`src/styles/markdown.css`):
   - Sizes: h1 1.3em, h2 1.15em, h3 1em.
   - h4–h6: 1em in `text-secondary`.
   - Margins: top 1.2em for h1/h2, 0.9em for h3.
   - Lists: `li + li { margin-top: .25em }`.
8. **Quieter footer.** Profile name at weight 500 in `text-secondary`.
9. **Scroll fade.** Add a 16 px bottom `mask-image` on `.messages` while not scrolled to the bottom, driven by the existing `pinned` flag in `MessageList.svelte`.
10. **Verify the blur.** Confirm the effect uses `EffectState::Active` and that `--bg-opacity` reaches CSS. Test over a bright wallpaper. If it still looks flat, lower the dark opacity to about .70.

**Accept when:**
- Compact empty has no space under the footer.
- Expanded: hairlines run edge to edge, content shares one left edge, there's no header, and headings are clearly tiered.
- Both default themes pass.

---

## Part B — TUI style

### B1. Settings & plumbing
- **New settings:**
  - `"style": "auto" | "minimal" | "tui"` (default `"auto"`). Rust: add it to `Settings` in `src-tauri/src/config/mod.rs`, with defaults in `config/defaults.rs`.
  - `"font": { "ui": string?, "mono": string?, "size": number? }` in settings. Fonts depend on the user's machine, so settings override a theme's `font`.
- **Theme schema:** a theme may declare `"style": "tui"` as its suggestion. `auto` resolves to the theme's suggestion, else `minimal`.
- **Applying the style:**
  - `applyTheme` in `src/lib/theme/apply.ts` sets `document.documentElement.dataset.style`.
  - Add `style` to `ResolvedTheme` and merge it in `mergeTheme`.
- **Palette:**
  - New action "Switch style" (cycles minimal ↔ tui), persisted via the existing settings patch helper, like `set_theme`.
  - **Live preview:** in the palette's theme list, moving the selection applies that theme temporarily. Esc restores the saved theme; Enter persists the selection.
- **CSS:** new `src/styles/tui.css`, with every rule scoped under `[data-style="tui"]` and imported in `main.ts`. Minimal CSS stays untouched. Split the file once it passes ~200 lines (e.g. `tui-markdown.css`).

### B2. New theme slot: `palette`
TUI decoration needs named hues. Add `palette` to the theme schema and flatten it to `--c-<name>`:
```json
"palette": { "red": "", "orange": "", "yellow": "", "green": "", "cyan": "", "blue": "", "purple": "", "pink": "" }
```
- Extend `mergeTheme` and `applyTheme`: `palette.x` → `--c-x`.
- Give `default-dark` and `default-light` palettes too, so every theme works in TUI style:
  - Dark: red `#ff6b6b`, orange `#fab387`, yellow `#f9e2af`, green `#a6e3a1`, cyan `#94e2d5`, blue `#89b4fa`, purple `#cba6f7`, pink `#f38ba8`.
  - Light: red `#d20f39`, orange `#fe640b`, yellow `#df8e1d`, green `#40a02b`, cyan `#179299`, blue `#1e66f5`, purple `#8839ef`, pink `#cc2f62`.
- **Profile colors** (both styles): a profile may set `"color": "<palette name>"`. Otherwise assign by index: purple, blue, green, orange, pink, cyan, yellow, red. The minimal footer dot uses it too.

### B3. `Frame.svelte` (new — the core TUI element)
A box with a title set into the top border, and an optional hint legend in the bottom border:
```
╭─ chats ────────────────╮
│ …content…              │
╰──────── ↵ open · p pin ─╯
```
- **Props:** `title`, `hint?` (bottom-right legend), `focused` (bool), `tone?` (palette name for the title color, default `accent`), `count?` (e.g. `12/48`, shown top-right as fzf does).
- **Build it from borders, not an opaque mask**, so blur stays visible behind the frame:
  - A top row of three flex children: left segment with `border-top` + `border-left` + top-left radius, the title text, and a fill segment with `border-top` + `border-right` + top-right radius.
  - The body carries the left, right and bottom borders plus the bottom radii.
  - The bottom legend uses the same technique, mirrored.
- **Minimal style:** `Frame` renders only its children. All decoration CSS lives in `tui.css`, and the title/hint elements get `display: none` outside TUI. Components always use `Frame`, so there's no branching in the markup.
- **Borders:**
  - 1 px, radius `--frame-radius: 6px`, which approximates `╭╮╰╯`.
  - Unfocused: `--border-strong`. Focused: `--accent`.
  - Title: `--text-tertiary` when unfocused; `--accent` and bold when focused. The title has 1 ch of padding on each side.
- **Focus follows the keymap context:** `keyContext()` in `ui.svelte.ts` decides which frame is focused (`composer` → ask frame, `sidebar` → chats frame, `palette` → commands frame). Keyboard-first becomes visible, like lazygit's active pane.

### B4. Layout in TUI style
- **Outer padding** 6 px, **gap between frames** 6 px, **window radius** 10 px.
- **Compact mode:**
  - Frame `ask · <profile>`: the composer, with a bottom hint `↵ send · ⇧⇥ profile · ⌃P cmds`.
  - Frame `<chat title>` above it, only when there are messages.
  - Status bar below.
- **Expanded mode:** frame `chats` on the left (240 px), frame `<chat title>` on the right, frame `ask · <profile>` below the thread, and a full-width status bar. The frame title replaces the header removed in A4.
- **Palette:** frame `commands` with `count` = matches/total. Settings: frame `settings`. Toast: small frame titled `error` / `warning` in the matching tone.

### B5. Typography
- Monospace everywhere: `[data-style="tui"] { --font-ui: var(--font-mono) }`.
- Base size 13 px, line height 1.5. The user can set `font.mono`, e.g. `"JetBrainsMono Nerd Font"`.
- Glyphs are ordinary Unicode (`❯ ▶ ● ◆ █ ┄`). No Nerd Font is required, and nothing depends on private-use glyphs.

### B6. Status bar (`StatusBar` look via `Footer.svelte` + `tui.css`)
Height 22 px, flush with the bottom, no border. Powerline arrows are drawn with CSS `clip-path` (no font needed):
```
 INSERT ▶ default ▶ deepseek-v4.1:floor          ◀ ↑35 ↓602 ◀ $0.0003 ◀ 🔒
```
- **Mode segment:** a solid colored background with `--bg` text, bold and uppercase. It shows the keymap context:

  | Mode | When | Color |
  |---|---|---|
  | `INSERT` | composer | `--c-green` |
  | `NORMAL` | sidebar | `--c-blue` |
  | `CMD` | palette | `--c-purple` |
  | `STREAM` | streaming | `--c-orange` |
  | `EDIT` | editing | `--c-yellow` |

- **Other segments:**
  - Profile: profile color on `--surface-hover`.
  - Model: `--text-secondary` on `--surface`.
  - Right side: usage, cost and sticky, as mirrored segments.
- **Key hints move out of the status bar** into the frame bottom legends (B3). In minimal style the Footer stays as it is.

### B7. Content styling in TUI
- **Messages:**
  - User: no bubble. A line prefixed `❯ ` in `--c-green`; text in `--text`, weight 600.
  - Assistant: a dim label line `◆ <model short name>` in `--c-blue` at `--fs-xs`, then the reply.
  - Separate turns with a dashed `border-top` in `--border` (`┄`).
- **Streaming:** a block caret `█` in `--accent`, blinking. The thinking label reads `◌ thinking 3s` in `--c-yellow`.
- **Markdown:**
  - Headings: h1 `--c-red`, h2 `--c-orange`, h3 `--c-yellow`, h4 `--c-green`. Optionally prefix h1/h2 with `# `/`## ` in `--text-tertiary` via `::before`.
  - Lists: bullets `•` in `--c-purple` via `::marker`; ordered numbers in `--c-blue`.
  - Inline code: `--c-green` on `--surface`. Links: `--c-blue`, underlined.
  - Blockquote: 2 px `--c-blue` bar. `strong` in `--text` bold; `em` in `--c-pink`.
  - `hr`: dashed. Tables: full grid in `--border-strong`, header text in `--accent`.
  - Code blocks: a mini `Frame` with the language as its title and `copy` as the bottom hint.
- **Lists (sidebar, palette), fzf-style selection:**
  - Selected row: `▶` pointer in `--accent`, `--surface` background, bold text.
  - Active chat: `--accent` text.
  - Palette kind shown as colored tags on the right: `[chat]` blue, `[model]` green, `[action]` purple, `[profile]` yellow, `[theme]` pink.
  - Matched characters in `--accent`.
- **Composer:** prompt `❯ ` in `--c-green` before the textarea. Caret: `caret-color: var(--accent)`, plus `caret-shape: block` where WebKit supports it.
- **Kbd in TUI:** no box, `--accent` text in angle-bracket style, e.g. `^P`, as lazygit shows keys.

### B8. Background
TUI works with or without blur. Community themes (Part C) default to `vibrancy: "hud"` and `opacity: .94` for the tinted-glass rice look. The user can set `opacity: 1`. Frames never paint an opaque mask (B3), so both work.

**Accept Part B when:**
- `style: "tui"` with `default-dark` looks like the B4 mockups.
- Switching style from the palette is instant and persists.
- The focused frame follows `Ctrl+H` / `Ctrl+L` / `Ctrl+P`.
- The mode segment changes with context and streaming.
- Blur is visible around and behind the frames.
- `no-raw-colors` test still passes, and minimal style is pixel-identical to before Part B.

---

## Part C — Community themes
Add these to `src/lib/theme/builtin/` (register in `index.ts`), each with `"style": "tui"`, `"vibrancy": "hud"`, `"opacity": 0.94`:
- `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe`, `catppuccin-latte`
- `gruvbox-dark`, `gruvbox-light`
- `nord` (rework the existing file)
- `tokyo-night`, `rose-pine`

The source palettes are below. Map them with the role tables; don't invent hex values.

### C1. Role mapping
`accent-soft` = accent at 16% alpha. `border` = surface0-equivalent; `border-strong` = overlay0-equivalent.

| Role | Catppuccin | Gruvbox | Nord | Tokyo Night | Rosé Pine |
|---|---|---|---|---|---|
| bg | base | bg0 | nord0 | bg | base |
| bg-solid / sidebar | mantle | bg0_h | `#272c36` | bg_dark | surface |
| surface | surface0 | bg1 | nord1 | bg_highlight | overlay |
| surface-hover / selection | surface1 | bg2 | nord2 | terminal_black | hl_med |
| border | surface0 | bg1 | nord1 | bg_highlight | hl_med |
| border-strong | overlay0 | bg3 | nord3 | terminal_black | hl_high |
| text | text | fg | nord6 | fg | text |
| text-secondary | subtext0 | fg3 | nord4 | fg_dark | subtle |
| text-tertiary | overlay1 | gray | `#7b88a1` | comment | muted |
| accent | mauve | orange | nord8 | blue | iris |
| accent-contrast | crust | bg0_h | nord0 | bg_dark | base |
| danger / warning / success | red / yellow / green | red / yellow / green | aurora red / yellow / green | red / yellow / green | love / gold / pine |
| code-bg | mantle | bg0_h | `#272c36` | bg_dark | surface |
| **palette** red, orange, yellow, green | red, peach, yellow, green | red, orange, yellow, green | aurora red, orange, yellow, green | red, orange, yellow, green | love, rose, gold, pine |
| **palette** cyan, blue, purple, pink | teal, blue, mauve, pink | aqua, blue, purple, purple | nord7, nord9, purple, purple | cyan, blue, magenta, magenta | foam, pine, iris, rose |

| Syntax | Catppuccin | Gruvbox | Nord | Tokyo Night | Rosé Pine |
|---|---|---|---|---|---|
| fg | text | fg | nord4 | fg | text |
| comment | overlay1 | gray | `#616e88` | comment | muted |
| keyword | mauve | red | nord9 | magenta | pine |
| string | green | green | green | green | gold |
| number | peach | purple | purple | orange | gold |
| title | blue | yellow | nord8 | blue | rose |
| built_in | yellow | orange | nord7 | cyan | foam |
| variable | maroon | blue | nord4 | red | text |
| meta | teal | aqua | nord10 | teal | iris |

### C2. Source palettes
**Catppuccin Mocha:**
- base `#1e1e2e`, mantle `#181825`, crust `#11111b`
- surface0 `#313244`, surface1 `#45475a`
- overlay0 `#6c7086`, overlay1 `#7f849c`
- text `#cdd6f4`, subtext0 `#a6adc8`
- red `#f38ba8`, maroon `#eba0ac`, peach `#fab387`, yellow `#f9e2af`, green `#a6e3a1`, teal `#94e2d5`, blue `#89b4fa`, mauve `#cba6f7`, pink `#f5c2e7`

**Catppuccin Macchiato:**
- base `#24273a`, mantle `#1e2030`, crust `#181926`
- surface0 `#363a4f`, surface1 `#494d64`
- overlay0 `#6e738d`, overlay1 `#8087a2`
- text `#cad3f5`, subtext0 `#a5adcb`
- red `#ed8796`, maroon `#ee99a0`, peach `#f5a97f`, yellow `#eed49f`, green `#a6da95`, teal `#8bd5ca`, blue `#8aadf4`, mauve `#c6a0f6`, pink `#f5bde6`

**Catppuccin Frappé:**
- base `#303446`, mantle `#292c3c`, crust `#232634`
- surface0 `#414559`, surface1 `#51576d`
- overlay0 `#737994`, overlay1 `#838ba7`
- text `#c6d0f5`, subtext0 `#a5adce`
- red `#e78284`, maroon `#ea999c`, peach `#ef9f76`, yellow `#e5c890`, green `#a6d189`, teal `#81c8be`, blue `#8caaee`, mauve `#ca9ee6`, pink `#f4b8e4`

**Catppuccin Latte** (light):
- base `#eff1f5`, mantle `#e6e9ef`, crust `#dce0e8`
- surface0 `#ccd0da`, surface1 `#bcc0cc`
- overlay0 `#9ca0b0`, overlay1 `#8c8fa1`
- text `#4c4f69`, subtext0 `#6c6f85`
- red `#d20f39`, maroon `#e64553`, peach `#fe640b`, yellow `#df8e1d`, green `#40a02b`, teal `#179299`, blue `#1e66f5`, mauve `#8839ef`, pink `#ea76cb`

**Gruvbox Dark:**
- bg0_h `#1d2021`, bg0 `#282828`, bg1 `#3c3836`, bg2 `#504945`, bg3 `#665c54`
- fg `#ebdbb2`, fg3 `#bdae93`, gray `#928374`
- red `#fb4934`, green `#b8bb26`, yellow `#fabd2f`, blue `#83a598`, purple `#d3869b`, aqua `#8ec07c`, orange `#fe8019`

**Gruvbox Light:**
- bg0_h `#f9f5d7`, bg0 `#fbf1c7`, bg1 `#ebdbb2`, bg2 `#d5c4a1`, bg3 `#bdae93`
- fg `#3c3836`, fg3 `#665c54`, gray `#928374`
- red `#9d0006`, green `#79740e`, yellow `#b57614`, blue `#076678`, purple `#8f3f71`, aqua `#427b58`, orange `#af3a03`

**Nord:**
- Polar night: nord0 `#2e3440`, nord1 `#3b4252`, nord2 `#434c5e`, nord3 `#4c566a`
- Snow storm: nord4 `#d8dee9`, nord6 `#eceff4`
- Frost: nord7 `#8fbcbb`, nord8 `#88c0d0`, nord9 `#81a1c1`, nord10 `#5e81ac`
- Aurora: red `#bf616a`, orange `#d08770`, yellow `#ebcb8b`, green `#a3be8c`, purple `#b48ead`

**Tokyo Night:**
- bg `#1a1b26`, bg_dark `#16161e`, bg_highlight `#292e42`, terminal_black `#414868`
- fg `#c0caf5`, fg_dark `#a9b1d6`, comment `#565f89`
- red `#f7768e`, orange `#ff9e64`, yellow `#e0af68`, green `#9ece6a`, teal `#73daca`, cyan `#7dcfff`, blue `#7aa2f7`, magenta `#bb9af7`

**Rosé Pine:**
- base `#191724`, surface `#1f1d2e`, overlay `#26233a`
- hl_med `#403d52`, hl_high `#524f67`
- muted `#6e6a86`, subtle `#908caa`, text `#e0def4`
- love `#eb6f92`, gold `#f6c177`, rose `#ebbcba`, pine `#31748f`, foam `#9ccfd8`, iris `#c4a7e7`

**Light themes** (Latte, Gruvbox Light): `scheme: "light"`, `vibrancy: "popover"`, and `accent-contrast` = base/bg0.

### C3. Tests
Add a Vitest in `src/lib/theme/` that loads every built-in theme and checks:
- Every theme resolves all `colors`, `syntax` and `palette` keys that `default-dark` defines.
- `text` on `bg` has contrast ≥ 4.5:1 and `text-tertiary` on `bg` ≥ 3:1. Write a small WCAG luminance helper in the test file; no dependency.

If a mapped color fails the contrast check, use the next brighter shade from the same palette instead (e.g. overlay2 instead of overlay1), and note it in the PR.

**Accept Part C when:**
- All nine themes appear in the palette, preview live, and look correct in both styles.
- The contrast test passes.
- A user theme `{"extends":"catppuccin-mocha","colors":{"accent":"#89b4fa"}}` works.

---

## Files (summary)
- **New:** `src/components/Frame.svelte`, `src/styles/tui.css` (+ a split file if needed), 9 theme JSONs, a theme contrast test.
- **Changed:**
  - `src/lib/theme/apply.ts` (style, palette, settings font override), `src/lib/types.ts`
  - `src/lib/state/theme.svelte.ts`, `src/lib/state/palette.svelte.ts` (style action, live preview)
  - `Footer.svelte`, `Sidebar.svelte`, `Palette.svelte`, `Thread.svelte`, `Composer.svelte`, `Message.svelte`, `Toast.svelte`, `Settings.svelte` (wrap regions in `Frame`)
  - `src/styles/base.css`, `src/styles/markdown.css`
  - `src-tauri/src/config/mod.rs` + `defaults.rs` (`style`, `font`), `src-tauri/src/panel/mod.rs` (`MIN_HEIGHT`)
- **Removed:** `src/components/Header.svelte`

## Verification
- Run `npm run check`, `npm test`, `cd src-tauri && cargo test && cargo clippy -- -D warnings`.
- Run `npm run tauri dev` and take screenshots of compact-empty, compact-with-reply, expanded, palette and settings for:
  - minimal + default-dark
  - minimal + default-light
  - tui + catppuccin-mocha
  - tui + gruvbox-dark
  - tui + catppuccin-latte
  - Each over a bright and a dark wallpaper.
- Walk through the keyboard flow in TUI: the focused frame and mode segment follow the context, and Esc restores the saved theme after previewing.
