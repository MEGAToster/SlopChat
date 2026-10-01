# SlopChat — Review Fixes & Design Plan

Two parts: fix the review findings (Part 1) before implementing the design spec (Part 2).


# Part 1 — Review fixes

## R. Review fixes — do these before new features
Found in a review of the state after commit `73a88d3` plus uncommitted work. Fix them in this order.

**Correctness**
1. **Stream is bound to a fake chat id.**
   - Problem: `CHAT_ID = "current"` in `src/lib/state/chat.svelte.ts`. Deltas are written into `state.messages[index]` of whatever chat is open, and `finalize` saves into the *currently active* chat. Switching chats mid-stream corrupts or misfiles the reply.
   - Fix:
     - Use the real chat id for `send`/`stop`.
     - Capture `chatId` and a local buffer at stream start.
     - Write deltas to the buffer and mirror them into the UI only while that chat is shown.
     - Persist to the captured `chatId`.
     - Key `Streams` in `providers/client.rs` by chat id and make `finish` remove only its own entry. Compare by `Arc::ptr_eq`.
2. **Settings write-back is lossy.**
   - Problem: `config::add_model/remove_model/set_theme` re-serialize the whole `Settings`. That drops unknown keys and formatting, writes every default into the file, and overwrites a file the user is mid-edit on (invalid JSON).
   - Fix:
     - Add one helper, `config::patch(app, |value: &mut serde_json::Value| …)`, that reads the file as `Value`, changes only the target key, and writes it back atomically (temp file + `rename`).
     - Refuse to patch while the file doesn't parse, and surface the error instead.
     - The three functions become one-liners.
3. **Hotkey can end up unregistered.**
   - Invalid `settings.json` at startup never registers the toggle (`config::init`). Register the default binding in that case.
   - `keybinds::apply` calls `unregister_all` before registering. If registration fails, re-register the previous shortcuts.
4. **Keybind collision:** `stop` and `scrollDown` both default to `Ctrl+D`. Make `stop` `Ctrl+Q` while streaming, and also Esc (see 9). Keep `Ctrl+D`/`Ctrl+U` for scrolling. Update both `config/mod.rs` defaults and `src/lib/keymap/defaults.ts`.
5. **Links navigate the webview away from the app.**
   - Intercept clicks on `a[href]` inside `.markdown` and call a new `open_url` command. It accepts only `http`/`https`/`mailto`, opens with `open`/`xdg-open`, and never uses a shell.
   - Set a real CSP in `tauri.conf.json`: `default-src 'self'; img-src 'self' data: https:; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost`.
6. **Hide can steal focus.**
   - Problem: `panel/macos.rs::hide` always calls `restore_frontmost()`. When the panel hides because the user clicked another app, this re-activates the *old* app.
   - Fix: only restore focus when SlopChat itself became active (e.g. after a native dialog). Never restore on the resign-key path.
7. **Composer focus is lost while streaming.**
   - Problem: `disabled={isStreaming()}` blurs the textarea and nothing refocuses it.
   - Fix: keep the textarea enabled (typing the next message is allowed), block only *send* while streaming, and refocus on `done`/`error`.
8. **Sync commands run on the main thread** (Tauri v2). Mark store, keychain and file commands `#[tauri::command(async)]` so SQLite or a keychain prompt can't freeze the UI.
9. **Esc chain from PLAN §3 isn't implemented.** Esc always hides. Implement: stop stream → close palette/settings → cancel edit → leave sidebar → collapse expanded → hide.
10. **Empty chats pile up.** `newChat()` inserts a row immediately. Make a new chat lazy: `activeChatId = null` until the first send (`ensureChat` already exists), and remove the `init()` auto-create.
11. **Provider errors inside SSE are dropped.** OpenRouter can send `{"error":{…}}` as a `data:` chunk with HTTP 200. Map it to `StreamEvent::Error`. Also emit a `Reasoning` state (from `delta.reasoning`) so the UI can show "thinking" instead of silence.
12. **Expanded mode uses the full screen size.** Use the monitor work area so the panel stays clear of the menu bar and Dock. Honor `window.compactWidth` instead of the `COMPACT_WIDTH` constant.

**Rules (CLAUDE.md)**
13. **Files over 300 lines:**
    - `App.svelte` (497): move key handling into `lib/keymap/handler.ts` and the layout into components.
    - `app.css` (636): see DESIGN.md §9.
    - `config/mod.rs` (381): move defaults to `config/defaults.rs` and the write helpers to `config/write.rs`.
    - `chat.svelte.ts` (315): move streaming to `state/stream.svelte.ts`.
14. **Hardcoded keys in `App.svelte`** (palette, filter and delete-confirm branches) bypass the dispatcher. Route everything through `resolve()` with new contexts `filter` and `confirm`, so all keys are configurable.
15. **Raw API keys in settings:** `providers::api_key` accepts a raw key from `settings.json`, which violates rule 5. Accept only `env:VAR`; anything else is an error.
16. **Provider duplication:** `openrouter.rs`, `ollama.rs` and `gemini.rs` are ~90% identical.
    - Replace them with one `providers/kinds.rs` table: default base URL, auth yes/no, models URL + parser, body extras (`usage.include` or `stream_options`), zero-cost flag.
    - Reuse the shared `reqwest::Client` with connect and read timeouts.
    - Fetch model lists concurrently with `tokio::join!`.

**Verify:** `cargo test`, `cargo clippy -- -D warnings`, `npm run check`, `npm test`. Then check manually:
- Switch chats mid-stream: the reply lands in the right chat.
- Break `settings.json`, then add a model: the file is untouched and an error is shown.
- Click a link in a reply: it opens in the browser.
- Click another app: the panel hides and focus stays on the clicked app.

---

# Part 2 — Design spec

# SlopChat — Design Spec (for an LLM coding agent)

Implement the visual design described here. Stay within CLAUDE.md: no UI or CSS libraries, small files, minimal comments.

**References:**
- **Raycast:** the launcher and AI chat. Quiet chrome, keyboard hints everywhere, neutral selection, translucent glass.
- **Cherry Studio quick assistant:** a compact assistant window with clean flat surfaces and a hint footer.

## 1. Principles
1. **Content first, chrome quiet.** Hairline dividers, no heavy borders, no gradients, no drop shadows inside the panel.
2. **Keyboard is visible.** Every actionable thing shows its keycap (`⌃P`, `↵`, `⎋`). Selection is always visible and never depends on hover.
3. **One accent, used sparingly.** Only for the focus ring, primary action, active-profile dot, streaming caret and links underline. Selection is neutral, not accent.
4. **Dense but calm.** 4 px spacing grid, 14 px base text, 40 px list rows in the palette and 32 px in the sidebar.
5. **Instant.** Keyboard navigation has no transitions. Motion only for things appearing or disappearing (≤150 ms).
6. **Everything themeable through tokens.** Components never contain raw colors.

## 2. Window & material
- **Defaults:**
  - Corner radius `--radius-window: 12px`, set in both `PanelBuilder.corner_radius` and CSS.
  - Native shadow on (`has_shadow(true)`).
  - A 1 px inner highlight: `box-shadow: inset 0 0 0 1px var(--border-strong)` on `.panel`.
- **Vibrancy (dark default is translucent):**
  - Apply a native effect with Tauri window effects in `panel/macos.rs` (`with_window(|w| w.effects(…))`).
  - Settings: material from the theme (`hud` | `popover` | `sidebar` | `none`), **`EffectState::Active`**, and radius = `--radius-window`.
  - The panel never becomes the active app, so the "follows window state" mode would render the inactive grey material.
  - The page background is `rgb(var(--bg-rgb) / var(--bg-opacity))` over the blur. With `opacity: 1` or `vibrancy: "none"`, it's a plain opaque panel.
- **Appearance sync:** the blur material follows the *window appearance*, not the CSS. When the theme changes, call `window.set_theme(Some(Dark|Light))` from the theme's `scheme` so the blur tint matches.
- **Linux:** `vibrancy` is ignored and `opacity` is forced to 1.

## 3. Theme format (single source of truth for colors)
The theme JSON is the only place colors live. `app.css` and components reference variables only. The panel is hidden until settings load, so the old CSS fallback copy isn't needed; delete it.

```json
{
  "scheme": "dark",
  "extends": "default-dark",
  "vibrancy": "hud",
  "opacity": 0.78,
  "colors": { "bg": "#18181b", "accent": "#f38ba8", "…": "…" },
  "syntax": { "keyword": "#cba6f7", "…": "…" },
  "font": { "ui": "…", "mono": "…", "size": 14 }
}
```
- `extends` defaults to `default-<scheme>`. Missing keys come from the base theme, so a user theme can override just `accent`.
- `apply.ts` flattens the theme into CSS custom properties:
  - `colors.x` → `--x`
  - `syntax.x` → `--syn-x`
  - `bg` also produces `--bg-rgb` (space-separated channels) for the opacity blend.
- Invalid values are skipped with one toast listing them. The theme still applies.
- Built-in themes in `src/lib/theme/builtin/`: `default-dark`, `default-light`, `nord`. Port `nord.json` to this schema.

### Color tokens
| Token | Default dark | Default light | Use |
|---|---|---|---|
| `bg` | `#18181b` (opacity .78, `hud`) | `#fafafa` (opacity .88, `popover`) | panel background |
| `bg-solid` | `#1f1f23` | `#ffffff` | toasts, menus over content |
| `sidebar` | `rgba(0,0,0,.15)` | `rgba(0,0,0,.03)` | sidebar tint |
| `surface` | `rgba(255,255,255,.05)` | `rgba(0,0,0,.04)` | inputs, user bubble, inline code |
| `surface-hover` | `rgba(255,255,255,.08)` | `rgba(0,0,0,.06)` | hover, keycaps |
| `selection` | `rgba(255,255,255,.10)` | `rgba(0,0,0,.08)` | selected row |
| `border` | `rgba(255,255,255,.08)` | `rgba(0,0,0,.08)` | hairlines |
| `border-strong` | `rgba(255,255,255,.14)` | `rgba(0,0,0,.14)` | window edge, code block border |
| `text` | `#ededef` | `#18181b` | primary text |
| `text-secondary` | `#a1a1aa` | `#52525b` | subtitles, blockquotes |
| `text-tertiary` | `#71717a` | `#8a8a93` | meta, section headers, hints |
| `accent` | `#f38ba8` | `#cc2f62` | see §1.3 |
| `accent-contrast` | `#1e1e2e` | `#ffffff` | text on accent fill |
| `accent-soft` | `rgba(243,139,168,.16)` | `rgba(204,47,98,.12)` | active pill bg, `::selection` |
| `danger` | `#ff6b6b` | `#d92d20` | delete confirm, errors |
| `warning` | `#f9e2af` | `#b45309` | config warnings |
| `success` | `#a6e3a1` | `#15803d` | "copied" |
| `code-bg` | `rgba(0,0,0,.28)` | `rgba(0,0,0,.04)` | code blocks |

Light mode uses a darker accent, `#cc2f62`, from the same hue as `#f38ba8`. `#f38ba8` on white is only about 2.3:1 contrast; `#cc2f62` is about 5:1.

### Syntax tokens (highlight.js classes map to `--syn-*` in `styles/syntax.css`)
| Token | Dark | Light |
|---|---|---|
| `fg` | `#ededef` | `#18181b` |
| `comment` | `#7f849c` | `#8c8fa1` |
| `keyword` | `#cba6f7` | `#8839ef` |
| `string` | `#a6e3a1` | `#40a02b` |
| `number` | `#fab387` | `#fe640b` |
| `title` (functions, classes) | `#89b4fa` | `#1e66f5` |
| `built_in` / `type` | `#f9e2af` | `#df8e1d` |
| `variable` / `attr` | `#eba0ac` | `#e64553` |
| `meta` / `tag` | `#94e2d5` | `#179299` |

### Non-color tokens (`styles/tokens.css`, not per-theme except `font`)
- **Font:**
  - `--font-ui: system-ui, -apple-system, "Inter", sans-serif`
  - `--font-mono: "SF Mono", ui-monospace, "JetBrains Mono", Menlo, monospace`
- **Size:** `--fs-xs 11px`, `--fs-sm 12px`, `--fs-md 14px` (base), `--fs-lg 15px` (composer), `--fs-xl 17px` (palette input). Line height 1.55 for prose, 1.3 for UI.
- **Weights:** 400 and 600 only.
- **Spacing:** `--sp-1…6 = 4 8 12 16 20 24px`.
- **Radius:** `--radius-sm 4px` (keycaps, inline code), `--radius-md 8px` (rows, code blocks, inputs), `--radius-lg 12px` (bubbles), `--radius-window 12px`.
- **Motion:** `--dur-fast 100ms`, `--dur 150ms`, `--ease cubic-bezier(.2,.8,.2,1)`. `prefers-reduced-motion` sets both durations to 0.
- **Numbers:** `font-variant-numeric: tabular-nums` for all numbers (tokens, cost).

## 4. Layout
Composer is **always at the bottom**. A footer action bar sits under it, as in Raycast.

### Compact (width = `window.compactWidth`, default 680)
```
┌──────────────────────────────────────────────────────┐
│  messages (scrolls; panel grows to max 480px total)  │
├──────────────────────────────────────────────────────┤ hairline
│  Ask anything…                                    ↵  │ composer, min 48px, 15px
├──────────────────────────────────────────────────────┤ hairline
│ ● default · sonnet-5    ↑1.2k ↓340 · $0.004   ⌃P Commands │ footer 36px
└──────────────────────────────────────────────────────┘
```
- **Empty chat:**
  - The message area collapses; the panel is just composer plus footer (~84 px), like a launcher bar.
  - When the composer is empty in a blank chat, show a single row of **profile chips** above the composer: the active chip uses `accent-soft` with an `accent` dot, and a `⇧⇥` keycap sits at the end. This is the Cherry quick-actions pattern.
- **Height:** follows content (existing `ResizeObserver` → `resize_panel`). Height changes aren't animated because the native resize is instant.

### Expanded (work area × `expandedScreenRatio`, centered)
```
┌──────────────┬───────────────────────────────────────────┐
│ ⌕ Filter   / │  Chat title                     ⌃⇧P sticky │ header 44px
│ PINNED       │───────────────────────────────────────────│
│ ▌Current chat│      messages, max-width 760px, centered   │
│ TODAY        │                                            │
│  Other chat  │                                            │
│ YESTERDAY    │───────────────────────────────────────────│
│  …           │      composer (same max-width)             │
│ + New chat ⌃J│───────────────────────────────────────────│
│              │  footer                                    │
└──────────────┴───────────────────────────────────────────┘
```
- **Sidebar:**
  - 240 px wide, `--sidebar` tint, `--border` right hairline.
  - Groups: Pinned, Today, Yesterday, Previous 7 days, Older. Section headers follow §5 Palette.
  - Chats sorted by `updated_at` desc within each group.
- **Header:** chat title in 14 px/600, truncated. Right side shows sticky state as an icon + keycap, tertiary when off and `accent` when on.

## 5. Components (each a Svelte file with scoped `<style>`)
- **`Kbd.svelte`** (new)
  - Look: inline-flex, height 18 px, min-width 18 px, padding 0 5 px, `--radius-sm`, background `surface-hover`, 1 px `border`, `--fs-xs`, `text-secondary`.
  - Input: a keybind string (`"Ctrl+Shift+P"`) or a list. Formatted by a new `keymap/format.ts` that reuses `parse.ts`.
  - macOS glyphs: `⌃ ⌥ ⇧ ⌘ ↵ ⎋ ⇥ ↑ ↓ ← →`. Linux uses words (`Ctrl`).
  - Always shows the *configured* binding, never a hardcoded one.
- **`Footer.svelte`** (new; replaces the `.meta`/`StatusLine` row)
  - Height 36 px, `border-top` hairline, padding 0 `--sp-3`.
  - Left: profile pill (accent dot + name in `text` + short model name in `text-tertiary`).
  - Middle: usage `↑in ↓out · $cost`, `text-tertiary`, tabular numbers. Last message, with the chat total on hover or in a title tooltip.
  - Right: contextual hints.
    - Idle: `⌃P Commands`.
    - Streaming: `⎋ Stop`.
    - Editing: `↵ Resend  ⎋ Cancel`.
    - Sidebar focus: `↵ Open  p Pin  d Delete`.
  - Stop is a keyboard hint; the only mouse target is the hint itself.
- **`Composer.svelte`**
  - Borderless textarea on the panel background, `--fs-lg`.
  - Auto-grows up to 8 lines, then scrolls. Placeholder in `text-tertiary`.
  - Trailing `↵` keycap, dimmed when empty.
  - Editing state: a 2 px `accent` left bar plus a small "Editing message" label.
- **`Message.svelte`**
  - **User:** right-aligned bubble, max-width 80%, `surface` background, `--radius-lg`, padding 8 × 12, plain text with preserved newlines.
  - **Assistant:** no bubble, full width, markdown (§6). A meta line under the last assistant message and on selection: `model · ↑↓ · $` in `--fs-xs text-tertiary`.
  - **Streaming:**
    - Before the first token: a `thinking… 3s` label in `text-tertiary` with a 1.2 s opacity pulse, driven by the `Reasoning` event.
    - While tokens arrive: a block caret `▍` in `accent` at the end, blinking at 1 s.
  - **Errors:** inline row with a `danger` icon, the message in `text-secondary`, and a `⌃R Retry` keycap if you add retry. Otherwise only the toast.
- **`MessageList.svelte`**
  - Spacing between messages is `--sp-4`.
  - Auto-scroll only if the list was at the bottom *before* the update: measure in `$effect.pre`.
  - Thin overlay scrollbar: 8 px, thumb `border-strong`, shown only while scrolling or hovering.
- **`Palette.svelte`** (Raycast root search)
  - Covers the panel. The background is the same material, so the blur shows through.
  - Input row: 52 px, `--fs-xl`, search icon in `text-tertiary`, hairline under it.
  - Sections in order: Suggestions (recent actions), Chats, Models, Profiles, Themes, Actions.
  - Section header: 28 px, `--fs-xs`/600, uppercase, letter-spacing .04em, `text-tertiary`.
  - Rows: 40 px, `--radius-md`, inset 8 px from the panel edges.
    - 16 px icon in a 24 px box, label `--fs-md`.
    - Matched characters in `text` 600, the rest `text-secondary`.
    - Right side: kind label (`Chat`, `Model`, `Action`) in `text-tertiary --fs-sm`, plus a `Kbd` if the action has a binding.
    - Selected row: `selection` background.
  - Palette footer bar (same component as Footer): left shows the breadcrumb (`Commands` / `Add model`); right shows `↵ Open`, `⎋ Back`.
  - Opens with a 100 ms fade plus translateY(4 px) → 0. Closes instantly.
- **`Sidebar.svelte`**
  - Rows: 32 px, `--fs-md`, truncated title.
  - Active chat: 2 px `accent` bar on the left. Selected (keyboard) row: `selection` background.
  - Pinned: a pin icon on the right.
  - Delete confirm: the row gets a `danger` tint at 12% alpha, the text changes to `Delete "…"?`, and `y`/`n` keycaps appear.
  - Filter input: the same style as the palette input, but 32 px and `--fs-md`.
- **`Toast.svelte`**
  - Bottom-center, 12 px above the footer.
  - `bg-solid` background, 1 px `border-strong`, `--radius-md`, shadow `0 8px 24px rgb(0 0 0 / .3)`, `--fs-sm`.
  - Leading icon colored by kind (danger/warning/success).
  - Auto-dismiss after 5 s. A persistent config error stays and adds `⌃P → Reload config`.
- **`Onboarding.svelte`** (inside the compact panel)
  - Title "Connect OpenRouter" at 15 px/600, then one line of secondary text.
  - Password input styled like the composer.
  - Footer hints: `↵ Save`, `⎋ Skip (use Ollama)`.
- **`Settings.svelte`**
  - Raycast preferences style: sections with headers (§5 Palette), rows 40 px, label on the left, control on the right.
  - Selects and inputs use `surface`, `--radius-md`, no visible border until focused.
- **`Icon.svelte`**
  - Keep inline SVGs, 16 px, stroke 1.5, `currentColor`, Lucide-style paths copied inline (no dependency).
  - Needed: search, chat, model, profile, theme, action, pin, trash, copy, check, enter, alert, stop, sidebar, expand, spark (thinking).

## 6. Markdown styling (`styles/markdown.css`, scoped under `.markdown`)
- **Text:** `--fs-md`, line-height 1.55, paragraphs spaced .6em.
- **Headings:** 600 weight, sizes 1.15/1.05/1em, margin-top 1em. No underlines.
- **Links:** `text` color with an underline in `accent` at 50% alpha (1 px, offset 2 px). On hover, the underline is full `accent`. Clicks go through `open_url` (fix R5).
- **Inline code:** `--font-mono` 0.9em, `surface` background, `--radius-sm`, padding 1 × 4.
- **Code blocks:**
  - `code-bg` background, 1 px `border`, `--radius-md`, horizontal scroll with no wrapping.
  - Header row, 28 px: language label (`--fs-xs`, uppercase, `text-tertiary`) on the left, copy button on the right.
  - The copy button is an icon that shows ✓ in `success` for 1.2 s after copying.
  - Code: `--font-mono` 12.5 px, line-height 1.5.
- **Lists:** 1.25em indent; markers in `text-tertiary`. Task-list checkboxes are drawn as small rounded squares, not native controls.
- **Blockquotes:** 2 px `border-strong` left bar, `text-secondary`, no italics.
- **Tables:** full width, hairline `border` cells, header row with `surface` background and 600 weight, `--fs-sm`, cell padding 6 × 10, horizontal scroll wrapper.
- **Rules and images:** `hr` is a `border` hairline. Images: max-width 100%, `--radius-md`.
- **Selection:** `::selection` background `accent-soft`.

## 7. Focus, states, accessibility
- **Focus ring:**
  - Standard: `:focus-visible` → `outline: 2px solid color-mix(in srgb, var(--accent) 60%, transparent); outline-offset: 1px`.
  - The composer shows no ring; the caret is enough.
  - List rows show selection through their background, not an outline.
- **Contrast:** body text at least 4.5:1 against `bg` composited over a mid-grey backdrop; tertiary text at least 3:1. Check both default themes with a small script or by hand, and record the results in the PR.
- **Non-color cues:** selection is never shown by color alone. The selected row also gets its trailing `↵` keycap.
- **Disabled:** 40% opacity with the cursor unchanged. Nothing is shown disabled that could instead be hidden.

## 8. Motion
- **Panel show/hide:** instant, as in Raycast.
- **Palette open, sidebar appear on expand, toast in:** `--dur` fade, plus a 4 px slide for toasts.
- **Selection moves and chat switches:** no animation.
- **Streaming caret blink and thinking pulse:** the only continuous animations. They stop when the panel is hidden, using `visibilitychange` or `panel://shown`/hidden events.

## 9. CSS architecture
Replace the 636-line `app.css`:
```
src/styles/
  tokens.css     non-color scales (§3)
  base.css       reset, html/body transparent, .panel background blend, scrollbars, focus, ::selection
  markdown.css   §6
  syntax.css     hljs classes → --syn-* (≈30 lines)
src/components/*.svelte   scoped <style> per component
```
- `main.ts` imports the four global files.
- No raw hex or rgba in components. Everything goes through a variable, which `npm run check` can enforce with a simple grep test in Vitest.
- Keep every file under 200 lines.

## 10. Implementation order & acceptance
1. **Theme engine:**
   - Tasks: the new schema, `extends` merging, flattening to variables, `--bg-rgb`, and port all three built-in themes.
   - Rust side: vibrancy with the Active state, plus appearance sync.
   - Accept when:
     - Switching `default-dark` ↔ `default-light` ↔ `nord` from the palette changes blur tint, colors and syntax instantly.
     - A user theme containing only `{"extends":"default-dark","colors":{"accent":"#89b4fa"}}` works.
2. **CSS split + `Kbd` + `Footer`.**
   - Accept when `app.css` is gone, no file is over 200 lines, and every hint shows the configured binding (change `palette` to `Ctrl+O` and the footer updates).
3. **Composer + messages + markdown styles.**
   - Accept when:
     - A reply containing headings, a table, a code block, a task list and links looks right in both themes.
     - Copy works.
     - The caret and thinking states show.
4. **Palette restyle with sections and match highlighting.**
5. **Sidebar with date groups, active bar and inline delete confirm.**
6. **Toast, onboarding, settings restyle.**

**Final check:**
- Take screenshots of compact-empty, compact-with-reply, expanded, palette and settings in both default themes, over a busy desktop wallpaper.
- The panel should read as glass in dark mode and stay legible.
- Idle CPU is 0% when the panel is hidden, meaning no animations are running.
