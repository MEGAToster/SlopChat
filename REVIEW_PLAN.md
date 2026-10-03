# SlopChat review — findings & fix plan

## Instructions for the executing agent
- **Repo:** `/Users/felixhalder/Code/projects/megatoaster/SlopChat`. Read `AGENTS.md` first and follow its rules:
  - files under 200 lines (split before 300);
  - comments say *why* and never describe history;
  - Tauri commands only validate and delegate;
  - platform code only in `panel/`/`platform/`;
  - no new dependencies unless unavoidable.
- **Order:** work batch by batch in order (1 → 7). Batch 7 depends on Batch 1 #6 (transactional migrations) and overlaps Batch 2 #17/#18. Before each fix, re-read the referenced code. Line numbers may have drifted, and items without ✔ must be confirmed before changing anything. Skip an item and note it if it turns out to be wrong.
- **After each batch:** run the verification commands at the bottom. Fix all failures before moving on.
- **Git:** do NOT commit or run any write git actions.
- **User decisions (final):**
  - Esc, Shift+Tab and Ctrl+H behave as intended. Change only the README/docs, never the code.
  - Keep `TODO.md` and `HISTORY_SYNC_PLAN.md` as they are.
  - Implement chat rename (Batch 6) and attachment cleanup (Batch 2 #23).
  - Do NOT implement delete-single-message, regenerate-earlier-message or export (export belongs to the future sync work).
- **Final report:** for each item, give fixed / skipped (with reason) / needs decision.

## Context
The user asked for a review of the code (overcomplication, dead code) and the design (inconsistencies, missing features).
- **Baseline:** `npm run check`, `npm test` (64 tests), `cargo clippy -D warnings` and `cargo test` (84 tests) all pass.
- **Rename:** no "rotation" leftovers.
- **Verification:** items marked ✔ were checked directly in the code.

The fixes are split into batches. Each batch can be done independently, in this order.

---

## Batch 1 — Security & data-loss bugs (do first)
| # | Problem | Where | Fix |
|---|---|---|---|
| 1 ✔ | **XSS:** a citation URL is put into `href="…"` unescaped, after DOMPurify has run. A search-result URL containing `"` can inject attributes into `{@html}`, which has access to Tauri IPC. | `src/lib/markdown/citations.ts:12`, `render.ts:133` | Escape the URL, or sanitize after `applyCitations`. |
| 2 ✔ | **SSRF parser mismatch:** the hand-rolled authority split differs from reqwest/WHATWG parsing (`http://127.0.0.1\@evil.com`). IPv6 `[::1]` only "works" by failing. | `src-tauri/src/tools/fetch_client.rs:216-245` | Parse with `reqwest::Url` and use `host()`/`port_or_known_default()`. |
| 3 | **SSRF gaps:** no check for IPv4-mapped IPv6 or `0.0.0.0/8`. Redirect hops are checked but not pinned (DNS rebinding). DNS lookups block the async runtime. | `fetch_client.rs:198-207, 258-275` | Use `to_canonical()`, follow redirects manually with a pinned client per hop, and use `tokio::net::lookup_host`. Update `docs/tools.md`. |
| 4 ✔ | **Agent log is always written to disk:** `Ring::flush()` writes when `live=false`, so prompts, queries and page content are logged even with the setting off. This contradicts docs/agent.md. | `providers/trace.rs:81`, `agent.rs:255,274` | Only write when the setting is on (or decide on "only on empty answer" and document it). |
| 5 ✔ | **Concurrent settings saves race:** read→edit→write with no lock and a fixed `.tmp` name, from async commands. | `config/write.rs:85-101, 275` | Hold a static `Mutex` across `patch_file`. |
| 6 ✔ | **Migrations are not transactional:** V3 (DROP/RENAME) can half-apply. | `store/migrations.rs:83-89` | Wrap each step and its `user_version` update in `conn.transaction()`. |
| 7 | **Message append/rewind are not atomic,** and rewind with an unknown id silently does nothing. | `store/messages.rs:89-170` | Use a transaction and return an error when 0 rows change. |

## Batch 2 — Functional bugs
| # | Problem | Where | Fix |
|---|---|---|---|
| 8 ✔ | **SSE breaks multi-byte UTF-8:** lossy decoding per chunk corrupts umlauts and emoji split across chunks, and the last event is dropped at EOF. | `providers/sse.rs:9` | Buffer bytes and decode complete lines only. |
| 9 ✔ | **Profile rename duplicates the profile:** no `previous` argument (unlike `saveModel`). Chats keep the stale profile name. | `Settings.svelte:100`, `config/write.rs:170-189` | Add `previous` end-to-end. |
| 10 ✔ | **Settings toggles do nothing** when the `search`/`debug` keys are missing. | `config/write.rs:243,260` | `entry().or_insert(json!({}))`. |
| 11 ✔ | **Provider error mid-research still runs the final pass:** the UI gets Error, then an answer. | `providers/agent/rounds.rs:66-69` | Return `Err(TurnError::Failed)`. |
| 12 | **Stop and the deadline don't cover `send()`:** Stop is ignored for up to 300 s while waiting for headers. | `providers/stream.rs:105-112` | Put `send()` inside the `select!`. |
| 13 ✔ | **Overwritten stream can't be stopped:** a new stream on the same chat overwrites the old `Cancel`, so the old stream runs on unstoppable. | `providers/client.rs:17-24` | Cancel the previous one on insert. |
| 14 | **Fallback model re-runs the paid pre-search.** | `providers/agent.rs:63, 136-143` | Compute the pre-search once and pass it in. |
| 15 | **Capability cache stores network failures** (e.g. Ollama down → tools off until restart). | `providers/capabilities.rs:97-172` | Cache only definitive answers. |
| 16 | **Wrong quota detection:** `mentions_quota` matches "limit"/"plan" in any error body and locks the provider for the month. The "allowances used" notice is one-shot and often wrong. | `tools/search/mod.rs:71, 219-226` | Match only 402/429 or explicit quota codes; notify only when all failures were `Exhausted`. |
| 17 | **Switching chats mid-stream drops the streaming placeholder:** the reply stays hidden until the chat is reopened. | `src/lib/state/chat.svelte.ts:114` | Reload messages on finish if the chat is active. |
| 18 | **Double Enter double-sends;** a failed send silently loses the draft. | `chat.svelte.ts:15`, `actions.svelte.ts:51` | Set a "sending" flag immediately; clear the draft only on success and report errors. |
| 19 ✔ | **"Editing message" indicator never shows** (props not passed). | `Thread.svelte:38`, `Composer.svelte` | Pass `editing`; drop the unused `placeholder`/`element` props. |
| 20 | **Auto-follow ignores tool steps** (the TODO.md item). | `MessageList.svelte:34-40` | Also track `last?.steps?.length`. |
| 21 | **Mermaid:** the copy button copies nothing, and the diagram theme follows the OS instead of the app theme. | `clicks.ts:8`, `render.ts:94`, `mermaid.ts:16` | Store the source in `data-source`; re-initialize on theme change. |
| 22 | **Smaller bugs:** search rank ordering (`ORDER BY MIN(rank)`); serper always forced last; `expanded_screen_ratio` unclamped; empty-URL Serper sources; `<svg/>` hides the rest of the page in html.rs; refusal text always says "web_search used up"; CITE line based on requested tools rather than offered ones; Done delayed by up to ~23 s by the generation-host lookup. | `store/search.rs`, `settings.rs:135`, `panel/mod.rs:78`, `serper.rs:59`, `html.rs:66`, `instructions.rs`, `prompt.rs:34`, `generation.rs` | Fix each one locally. |
| 23 ✔ | **Attachment files leak:** deleting a chat (`store/chats.rs:135`) or rewinding (`store/messages.rs` `rewind`) cascades the `attachments` rows, but the files in `paths::attachments_dir()` stay on disk. | `store/chats.rs`, `store/messages.rs`, `store/attachments.rs` | (1) Before the delete, `SELECT path` of the affected attachment rows in the same transaction. (2) Commit. (3) Then remove the files best-effort, ignoring NotFound. Put the helper in `store/attachments.rs`. (4) Also run a one-off orphan sweep in `Store::open`: delete files in `attachments_dir()` that no row references. Add tests. |

## Batch 3 — Design inconsistencies (README vs behaviour)
Items 1–3: the code is correct; update `README.md` (keybind table) and any `docs/` mentions to describe the actual behaviour. Read the code to get the exact wording right.
1. ✔ **Esc:** the README claims "Stop → close → leave sidebar → collapse → hide". In reality:
   - palette, settings and the sidebar filter close on Esc;
   - otherwise Esc hides the panel (`escChain` in `overlays.svelte.ts`, sidebar Esc in `dispatch.ts`).
2. **Shift+Tab:** cycles the profile whatever the composer contains (`dispatch.ts:109`). Drop "(composer empty)".
3. **Ctrl+H:** in compact mode it expands the sidebar and keeps focus in the composer (`expandSidebar`, `overlays.svelte.ts:115`); in expanded mode it collapses. Rewrite the README row to match.
4. **"New chat"** shows the Ctrl+J hint, but Ctrl+J only creates a chat when you're on the last one (`paletteItems.ts:55`, `Sidebar.svelte:113`).
5. **New profiles** default to a `useContext: true` override instead of inherit (`Settings.svelte:86`).
6. **Store registered after the window:** an early IPC call can fail. The in-memory fallback is silent, so the user isn't told history won't persist (`main.rs:105-118`).
7. **Startup error text** says "using previous settings" when the app is actually on defaults (`config/mod.rs:205`).
8. **Doc drift:**
   - wrong path `providers/agent/trace.rs`;
   - content-type check claimed but absent (PDFs and images are decoded as text);
   - 50-char minimum applies only with reader fallback on;
   - per-hop IP pinning claimed but absent.

**Out of scope (user decision):** delete single message, regenerate earlier message, export (later, as part of sync).

## Batch 4 — Dead code
- **Tauri commands never invoked:** `show_panel`, `toggle_panel`, `toggle_panel_expanded`, `is_panel_expanded` (`main.rs:47-58`), and the now-unused `panel::toggle_expanded`. ✔
- **`fetch_url` alias:** config already renames it on load; remove it from `tools/mod.rs`, `agent/events.rs`, `chunks.rs`, `Message.svelte:76` and the docs.
- **Rust:**
  - `RawSearchSettings.provider`;
  - `default_style` and `default_true` serde defaults (redundant with `serde(default)`);
  - `impl Default for Cancel`;
  - `#[serde(default)]` on a serialize-only enum (`providers/mod.rs:125`);
  - the `ToolOutcome` copy of `ToolOutput`;
  - `check_text`'s always-true `reader_tried` parameter.
- **Frontend:**
  - `ui.svelte.ts` `mode()`;
  - `SendRequest.webToggle`;
  - the unreachable `"CMD"` footer mode;
  - the unreachable sidebar `Ctrl+L` branch;
  - `.divider` in `tui.css`;
  - tokens `--fade-bottom`, `--sp-5`, `--sp-6`.
- **History comment** at `MessageList.svelte:66-67` (rule violation).

## Batch 5 — Overcomplication / file size
- **Over 300 lines — split:**
  - `Settings.svelte` (645) → one section component each, plus a data-driven `NumberRow` replacing 9 copy-pasted `adv-row`s;
  - `config/settings.rs` (513) → move legacy handling into `legacy.rs`;
  - `config/write.rs` (349);
  - `providers/agent.rs` (330);
  - `Message.svelte` (321) → extract the steps list;
  - `commands.rs` (311);
  - `tools/fetch_client.rs` (310) → move the SSRF guard into `tools/ssrf.rs`.
- **Thin-layer violations:**
  - commands with logic: `store_api_key`, `model_reasoning`, `get_chat`, `initial_chat`;
  - `#[cfg(target_os)]` code in `main.rs` → move to `platform::`.
- **Pass-through wrappers:**
  - 13 forwarding fns in `config/mod.rs` → `pub use write::{…}`;
  - `cli::run`/`cold_start` merge into one;
  - `Sticky` merges into `PanelState`;
  - `NewAttachment` and `Attachment` become one struct;
  - `Turn::with_deadline` → struct update syntax.
- **Duplication:**
  - search defaults written three times in `settings.rs`/`defaults.rs`;
  - concurrent vs sequential tool branches in `rounds.rs:119-186`;
  - OpenRouter model list fetched twice (`capabilities.rs` vs `Kind::fetch_models`);
  - web and reasoning per-chat override logic (`chatStore` vs `reasoning`);
  - `openPalette`/`openModels`;
  - `restoreComposerFocus`/`focusComposerSoon`;
  - Tab detection in both `handler.ts` and `dispatch.ts`.
- **Stateless factories:** `createOverlays`/`createActions` → plain module exports. `.svelte.ts` files that use no runes → `.ts`.
- **Naming:**
  - `tools/budget.rs` → `tools/search/allowance.rs` (clashes with `agent/budget.rs`);
  - settings struct `Debug` → `DebugSettings`;
  - `db_err` is also used for filesystem errors.
- **CSS:**
  - `.control` defined three times;
  - ApiKeyRow/Onboarding/Settings re-implement `.row`/`.btn`/`.primary`;
  - duplicate palette/settings frame blocks in `tui-app.css`.

---

## Batch 6 — Feature: rename chat
- **Goal:** let the user give a chat a custom title. Today titles are only set automatically through `store/chats.rs::set_title_if_empty`.
- **Backend:**
  - add `chats::rename(conn, id, title)` in `store/chats.rs` (trim, reject empty, cap the length, error when 0 rows are updated);
  - add a `Store` method;
  - add a thin `rename_chat` Tauri command in `commands.rs` and register it in `main.rs`.
- **Frontend:**
  - add `renameChat` to `src/lib/ipc.ts`;
  - add a `renameChat` action to the sidebar dispatch, next to `pinChat`/`deleteChat` (`src/lib/keymap/dispatch.ts:159-160`). Use key `r` and confirm it doesn't clash;
  - do the rename inline in `Sidebar.svelte`: the row title becomes an input, Enter saves, Esc cancels. Reuse the existing pending-delete/filter state pattern in `ui.svelte.ts`;
  - add a "Rename chat" palette item (`paletteItems.ts`) that acts on the active chat;
  - refresh the chat list after renaming.
- **Docs:** add `r` rename to the README sidebar keys line.
- **Tests:**
  - Rust: rename, empty title rejected, unknown id → error;
  - dispatch test for `r` in sidebar context (`src/lib/keymap/dispatch.test.ts`).

## Batch 7 — Palette-picked model (user-reported bugs + per-chat persistence)
This batch covers a model chosen from the models palette that isn't the active profile's model. Today it is a session-only `store.modelOverride` (`src/lib/state/ui.svelte.ts:26,83-88`), and `activeConfig()` (`ui.svelte.ts:171`) resolves it.

### 7a. Bug: an override is shown as if it were the profile
- **Symptom:** the footer shows the profile dot, colour and name ("● default mimo-v2.6-pro:floor"), and the composer frame shows "ask · default".
- **Where:**
  - `Footer.svelte` (`profile`/`color`/`.name` derive from `activeProfile()` only);
  - `Thread.svelte` (`title={\`ask · ${profileName}\`}`).
- **Fix (user choice: model only, neutral):**
  - export one helper from `ui.svelte.ts`, e.g. `overrideActive(): boolean` (true when `modelOverride()` is set and differs from `activeProfile().model`);
  - when it's true:
    - the footer hides the profile dot and name, shows only the model (`model` + tag) in a neutral tone (no `--c-<profile colour>`), and keeps the effort/web controls;
    - the composer frame title becomes `ask · <short model id>` (same short form as the footer: last path segment + tag);
  - use only existing theme tokens; `src/lib/theme/no-raw-colors.test.ts` must still pass.

### 7b. Bug: sending does nothing in an existing chat after picking a non-profile model
- **Symptom (user):** Enter does nothing. There is no error, and the chat had previously been answered by another model.
- **Not yet reproduced.** The backend errors are already caught in `stream.svelte.ts` (`run` → try/catch → shown as ⚠). So the silent no-op must come from the frontend send path:
  - `actions.svelte.ts` `handleSend` → `void submit(...)`;
  - `chat.svelte.ts` `submit`/`streamReply`.
- **Steps:**
  1. Reproduce with `npm run tauri dev`:
     - open an existing chat with history;
     - Ctrl+M → pick a registered model that is in no profile;
     - type and press Enter.
  2. Add temporary `console.error` logging to find the cause. Candidates, most likely first:
     - `chatState.streaming` is still `true` from an earlier/other-chat stream, so `submit` returns early (`chat.svelte.ts:15`). `streaming` is global, not per chat; the "switching chats mid-stream" bug (#17) leaves it inconsistent;
     - an exception in `submit` (e.g. `appendMessage`, `messageContent`) is swallowed by `void handleSend()`, and the draft was already cleared (#18);
     - key context is not `composer` after the palette closes, so Enter never dispatches `send` (`overlays.svelte.ts` `closePalette`/`restoreComposerFocus`, `keymap/dispatch.ts`);
     - the backend `send` never returns (e.g. a capability or OpenRouter lookup for an unknown model with no timeout, `providers/capabilities.rs`). Check whether the footer shows STREAM.
  3. Fix the root cause, not the symptom.
- **Also, independent of the root cause** (overlaps #18 — do once):
  - every send path surfaces errors through `reportError`;
  - the draft is cleared only after `submit` succeeds;
  - an early return in `submit` caused by streaming in another chat shows a notice instead of doing nothing.
- **Remove all temporary logging afterwards.**

### 7c. Feature: persist the picked model per chat (user choice)
- **DB:**
  - migration V5 in `src-tauri/src/store/migrations.rs` adds `chats.model TEXT` (nullable; NULL = use the profile's model). Use the transactional migration from Batch 1 #6;
  - add `model` to `Chat` in `store/chats.rs` (`COLUMNS`, row mapping) and to `src/lib/types.ts`;
  - add `chats::set_model(conn, id, Option<&str>)`, a `Store` method and a thin `set_chat_model` command; register it in `main.rs` and `src/lib/ipc.ts`. Mirror the existing `setChatProfile` path end to end.
- **Frontend (`ui.svelte.ts`):**
  - **picking a model** in the palette (`palette.svelte.ts` `case "model"`) sets the override and, if a chat is active, persists it. On a blank chat (no id yet), keep it as a draft and persist it on creation. Follow the existing draft pattern: `adoptWebDraft`/`adoptReasoningDraft` called in `submit`; add an `adoptModelDraft(chatId)` alongside;
  - **picking the profile's own model** clears the override (store NULL);
  - **opening or previewing a chat** restores the override from `chat.model` instead of setting it to `null` (`openChatWithProfile`, `previewChatWithProfile`, `syncProfileFromActive`). Run `syncProfileFromActive()` first, then apply `chat.model`;
  - **`cycleProfile` / `setProfile`** clear the override and persist NULL for the active chat (profile choice wins);
  - **a new chat** starts with no override.
- **Stale models:** if `chat.model` is no longer in `settings().models`, still use it. `activeConfig` already falls back to `{ model: ref, useContext: true }`. Show it as an override per 7a.
- **Tests:**
  - Rust: migration V5 on an existing DB; `set_model` round trip; NULL clears;
  - vitest: `activeConfig`/`overrideActive` for profile model vs override, if it's testable without Tauri (see existing `*.test.ts` patterns).
- **Docs:** add a README line under Profiles: "A model picked from the palette overrides the profile for that chat and is remembered per chat; cycling the profile clears it."

## Verification
After each batch:
- **Automated checks:** `npm run check && npm test`, plus `cd src-tauri && cargo clippy -- -D warnings && cargo test`.
- **New unit tests:**
  - SSE split UTF-8 (`sse.rs`);
  - SSRF cases `\@`, `[::1]`, `::ffff:127.0.0.1`;
  - citation URL containing `"`;
  - profile rename;
  - toggles with missing keys;
  - migration rollback.
- **Manual (`npm run tauri dev`):**
  - rename a chat from the sidebar (`r`) and from the palette;
  - pick a non-profile model in an existing chat:
    - footer and composer show only the model, in a neutral tone;
    - Enter sends;
    - switch chats and back → the override is restored;
    - Shift+Tab → the override is cleared;
  - delete a chat with an image attachment → its file is gone from the attachments dir;
  - stop during a slow first token;
  - switch chats mid-stream;
  - agent log off → `~/.local/share/SlopChat/logs/agent.jsonl` not written;
  - rename a profile.
