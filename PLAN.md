# Implementation plan: feats.md

Self-contained plan for an implementing agent. Read `AGENTS.md` first: its rules (lightweight,
<200-line files, why-only comments, thin Tauri commands, platform code only in `panel/`/`platform/`,
no secrets on disk, no speculative features) apply to every step.

**Per step:** implement → add tests → run all checks → remove the shipped item from `feats.md`.

```
cd src-tauri && cargo test && cargo clippy -- -D warnings
npm test && npm run check
```

Work the steps **in order**. Steps 7–9 depend on 6 (SearXNG) and on step 1 (toasts).

## Decisions already made (do not re-litigate)

- Web modes become **Off / Auto / Deep**. `Always` is removed (old configs load as `Auto`).
- Deep research warns (toast) when a search provider other than SearXNG is or would be used. It
  does not block.
- With an `antigravity/*` model selected, Deep research runs **only** inside `agy`. No SlopChat search
  providers, budgets or subagents are used.
- Deep research uses cheap subagents when `advanced.deep.subagentModel` is set; otherwise single loop.
- OpenRouter "tools" option = only route to providers that support tools (already implemented via
  `require_parameters`, must be preserved). "Max output filter" is **dropped**.
- SearXNG is a new keyless search provider (no prior code exists in git history).

---

## Step 1 — Toast fixes

Files: `src/components/Toast.svelte`, `src/App.svelte`, `src/lib/state/ui.svelte.ts`,
`src/lib/state/palette.svelte.ts`, `src/styles/tui*.css`.

1. **Chat errors render as "notice" (info, spark icon).** `App.svelte:73` calls `notify(error)` with
   the default kind. Pass `"danger"`. Also `palette.svelte.ts:261` ("no API key") → `"warning"`.
2. **Double box in TUI.** `.toast` has its own border/radius/padding/background *and* wraps `Frame`,
   which draws edges in TUI. Under `[data-style="tui"]`, drop the toast's own box (keep a solid
   background) and let the frame draw it.
3. **Tone never shows.** `Frame` only applies `--frame-tone` to the title when `focused`
   (`tui.css:59`). Make the toast frame use its tone for edges and title (pass `focused` or add a
   toast-scoped rule).
4. **Config error hides all toasts.** `App.svelte:266` uses `{#if persistentError}…{:else if
   uiNoticeText}`. Render the config error as its own pinned banner so normal notices still show
   (Step 7's Deep warning depends on this).
5. **Repeated identical message doesn't restart the 5 s timer.** The effect at `App.svelte:76`
   keys on the string. Add a notice counter/id in `ui.svelte.ts` that changes on every `notify`.
6. **Overflow:** add `overflow-wrap: anywhere`. Clamp to ~4 lines.
7. **A11y/dismiss:** `role="status"` (`role="alert"` for danger), `aria-live`, click to dismiss.
8. **Cleanup:** replace the hardcoded `bottom: calc(36px + …)` with a footer-height token, and the
   three kind classes (`warning`/`success`/`.icon.info`) with one `data-kind` attribute.

Verify visually in both styles (minimal + tui) at narrow and wide panel widths.

## Step 2 — Other UI fixes

Found by a code audit. Items marked ✓ were confirmed by reading the code; re-verify the rest before fixing.

**TUI style clashes**
1. ✓ **Sidebar rows hit by frame CSS.** Generic selectors `.frame .title`, `.frame .body`,
   `.frame .line` (`src/styles/tui.css:53-75,114-125`) match unrelated descendants. Scope them to
   the frame's own parts (e.g. `.frame > .edge .title`, `.frame > .body`, `.frame > .edge .line`).
   This also fixes:
   - sidebar row titles (`Sidebar.svelte:125,128`) getting padding, grey colour, and bold accent on
     every row when focused, so the active row is indistinguishable;
   - ✓ Settings' inner `<div class="body">` (`Settings.svelte:177`) getting a second pair of side
     borders;
   - the Antigravity path `.line` (`AntigravitySection.svelte:33,35`) being centered.
2. ✓ **Duplicate Settings header in TUI.** The h2 "Settings" and "Done" bar (`Settings.svelte:172-175`)
   sit inside a frame already titled "settings". Hide them in TUI like the palette's `.bar`
   (`tui-app.css:94`).
3. **Prompt glyph on its own line.** The `❯`/`◆` glyphs (`tui-app.css:16-26`) render on their own
   line because `.message.user` is a flex column (`Message.svelte:134`).
4. **Frame title/hint overflow.** Frame title, hint and count are `nowrap` with no shrink. Add
   `min-width: 0` and an ellipsis so long chat titles or model names don't push the corner out.

**Overflow**
5. ✓ **Transcript overflow.** `markdown.css` has no `overflow-wrap`, and tables are `width:100%`
   with no scroll wrapper (`markdown.css:189`). Long URLs and wide tables overflow sideways. Add
   `overflow-wrap: anywhere` and a horizontally scrolling table wrapper (in the markdown renderer or
   CSS). Same for long targets in `MessageSteps.svelte:80`.
6. **Attachment chips in sent messages** (`Message.svelte:197-207`): add an ellipsis, matching the
   Composer chip.
7. **Long profile names.** In the footer (`Footer.svelte:263`, `.name`) use nowrap + ellipsis. In
   Settings (`Settings.svelte:533`, `.item-name` is `flex:none`) let it shrink.

**State / labels**
8. ✓ **Rename lost on blur** (`Sidebar.svelte:107`, `onblur → setRenaming(null)`). Save on blur
   instead; Esc still cancels.
9. **Antigravity status stuck on "Checking…"** after `antigravityStatus()` throws
   (`AntigravitySection.svelte:14-15,38`). Use a separate error state.
10. **Hardcoded keybind text.** "Profiles (Shift+Tab)" (`Settings.svelte:213`) should come from the
    `cycleProfile` binding via `formatKeys`. The effort tooltip (`Footer.svelte:103`) should use
    `formatBinding` and handle the unbound case (currently "( to cycle)").
11. **Inconsistent Antigravity heading.** `AntigravitySection.svelte:56-63` redefines
    `.section-title`; use the shared one (`editor.css:4`).

**Low-impact batch**
- `aria-label` on icon-only buttons: Footer follow/sticky/web (`Footer.svelte:127-159`), copy and
  retry (`Message.svelte:103,107`), attach (`Composer.svelte:110`), key remove (`ApiKeyRow.svelte:76`).
- Wrap Settings inputs/checkboxes in `<label>` (`Settings.svelte:245-462`, `ApiKeyRow.svelte:64`).
- Hover highlight on palette rows (`Palette.svelte:164-183`).
- Composer drag outline flickers when the pointer is over the textarea (`Composer.svelte:86`);
  ignore `dragleave` when `relatedTarget` is inside the box.
- Hardcoded `13px` (`Sidebar.svelte:233,264,304`) and `12.5px` (`markdown.css:90,168`) → tokens.
- Wording: "Search chats" vs "Filter" (`Sidebar.svelte:83,90`). "Keychain" in `Onboarding.svelte:33`
  is wrong on Linux; use "OS keychain".

`Settings.svelte` is 648 lines (rule: split before 300). While touching it here or in Steps 4 and 8,
move sections into components (`GeneralSection.svelte`, `SearchSection.svelte`) like
`AntigravitySection.svelte`.

## Step 3 — Chat order follows last edit, not creation

The resume timeout (`store/chats.rs::initial/within`) **already** uses `updated_at`, which
`messages::append` and `messages::rewind` update. The bug is ordering:

- `store/chats.rs::list` sorts by `created_at ASC`. The frontend relies on that order:
  - `chatHistory.svelte.ts::step`: Up on a blank chat opens `list[last]`, the newest *created*
    chat, not the last edited one.
  - `removeChat` falls back to `list[last]`.
  - Prev/next order differs from the sidebar, which groups by `updatedAt` (`chatGroups.ts`).

Steps:
1. `chats::list`: `ORDER BY updated_at ASC, created_at ASC`. Check `store/search.rs` for a
   consistent order.
2. Tests in `store/mod.rs`:
   - `list` returns the last edited chat last;
   - a rewind moves its chat to the end.
3. Only message activity counts as an edit. Rename, pin and model changes do **not** touch
   `updated_at`.

## Step 4 — Setting: convert long pastes to a file (0 = never)

Text attachments are already inlined as `File: <name>\n\n<content>` by
`chat.svelte.ts::messageContent`, so only the composer needs to change.

1. **Config:** `pasteAsFileChars: u64` in `config/settings.rs` (default `0`) and `defaults.rs`.
   Wire it like `resumeWithinMinutes`: `write.rs::set_paste_as_file`, `commands.rs` (validate +
   delegate), `main.rs` handler list, `ipc.ts`, `types.ts`, `settings.svelte.ts` default.
2. **UI:** a number input next to "Resume within" in Settings (inside the new `GeneralSection.svelte`).
3. **`Composer.svelte::onPaste`:** when no files were pasted, the setting is > 0, and
   `text.length >= setting`:
   - `preventDefault()`;
   - `addFiles([new File([text], "paste-<n>.txt", { type: "text/plain" })])`.
4. **UTF-8 bug (must fix):** `chat.svelte.ts::decode` uses bare `atob`, which mangles non-ASCII.
   Decode bytes with `new TextDecoder().decode(Uint8Array.from(atob(data), c => c.charCodeAt(0)))`.
5. Text chips show a size hint, e.g. `paste-1.txt · 12k`.
6. Vitest:
   - threshold logic (0 never; below, at and above the limit);
   - a UTF-8 round trip through `decode` (umlauts and emoji).

Put the threshold check in a small pure helper so it is testable.

## Step 5 — Images from the clipboard

`Composer.svelte::onPaste` only reads `clipboardData.files`. WKWebView/WebKitGTK often leave it
empty for screenshots and browser-copied images.

1. Iterate `clipboardData.items`, `getAsFile()` for every `kind === "file"`. Use `files` only as a
   fallback. Deduplicate.
2. Name fallback: an empty `file.name` → `pasted-<timestamp>.<ext from mime>`, so
   `store/attachments::save` gets an extension.
3. Manually verify on macOS and Linux:
   - a screenshot to the clipboard (Cmd+Ctrl+Shift+4);
   - an image copied in a browser;
   - an image file copied in Finder or a file manager.
4. **Only if** step 3 shows WebKit still provides nothing: add a native fallback.
   - `read_clipboard_image` command → `platform/` code (NSPasteboard on macOS, GTK clipboard on
     Linux) returning base64 PNG.
   - Avoid `tauri-plugin-clipboard-manager` unless the native code is clearly larger.

## Step 6 — OpenRouter routing options

`max_completion_tokens` per model already exists (`ModelConfig` → `Limits` →
`providers/kinds.rs::apply_limits`); just make sure it's visible in `ModelEditor`.

**Note:** `kinds.rs::body` currently *assigns* `body["provider"] = { require_parameters: true }`
when tools are sent. Routing must **merge** into that object and keep `require_parameters`.

1. **Config:** in `config/settings.rs` `ModelConfig`, add
   `routing: Option<Routing>` (serde default, `skip_serializing_if = "Option::is_none"`):
   ```
   Routing { order: Vec<String>, only: Vec<String>, ignore: Vec<String>,
             allow_fallbacks: Option<bool>, sort: Option<String /* price|throughput|latency */>,
             max_price: Option<MaxPrice { prompt: Option<f64>, completion: Option<f64> }> } // USD per 1M tokens
   ```
   Empty vectors and `None` are omitted from the request.
2. **Request:** new `providers/routing.rs` with `pub fn apply(body: &mut Value, routing: &Routing)`.
   It is OpenRouter-only and merges into `body["provider"]`. Call it next to `apply_limits`
   (`providers/agent/turn.rs:73`). The backend reads routing from the model config, so the IPC
   request does not change. (`kinds.rs` is 251 lines; don't grow it.)
3. **UI:** `RoutingEditor.svelte`, collapsible inside `ModelEditor.svelte`, only for `openrouter/`
   models. Provider lists are comma-separated inputs; two price inputs; sort select;
   allow-fallbacks checkbox.
4. Tests:
   - tools + routing merge keeps `require_parameters`;
   - empty routing adds no `provider` key;
   - serde round trip in `models_file.rs`.
5. Docs: routing section in `docs/limits.md`.

## Step 7 — SearXNG search provider

Pattern: `src-tauri/src/tools/search/brave.rs` and `dispatch` in `tools/search/mod.rs`.

1. **`tools/search/searxng.rs`:**
   - Request: `GET {baseUrl}/search?q=<q>&format=json`.
   - Parse `results[]` (`title`, `url`, `content` → snippet, `publishedDate` → published) into `Hit`s.
   - Map `answers[]` and `infoboxes[]` to URL-less hits; `format_output` already treats those as
     context only.
   - `cost_usd = 0`, `units = 1`.
   - Respect `n` by truncating.
2. **`dispatch`:** add `"searxng" => searxng::search(...)`.
3. **Keyless:** `run()` currently skips providers without a key (`search_key`). Add
   `fn needs_key(name) -> bool` (false for searxng). For searxng, require a base URL instead.
   Missing URL → reason `searxng: no base URL`.
4. **Config:**
   - add `search.searxng { baseUrl: String }` to `SearchSettings`;
   - **also** add it to `config/legacy.rs::RawSearchSettings`, because `SearchSettings`
     deserializes via `from`;
   - add the default in `defaults.rs`.
5. **Error hint:** HTTP 403, or a non-JSON (HTML) body, → `Failed("SearXNG JSON output is disabled
   — add `json` to search.formats in settings.yml")`.
6. **UI** (`SearchSection.svelte`, split out of Settings):
   - a SearXNG row with a base URL input instead of `ApiKeyRow`;
   - include `searxng` in the provider order list and the order hint (`Settings.svelte:46-52`).
7. Tests:
   - parse a fixture of SearXNG JSON (results + answers + infobox);
   - missing base URL;
   - 403/HTML classification.
8. **Docs** (`docs/tools.md` → Providers):
   - `docker run searxng/searxng` snippet;
   - enabling the JSON format;
   - public instances usually disable JSON, and upstream engines rate-limit bursts, so self-host.

## Step 8 — Web modes Off / Auto / Deep + single-loop Deep + paid-search warning

### Mode model
1. **Backend:** `config/settings.rs` `WebSearchMode { Off, Auto, Deep }`. Old `"always"` must
   deserialize as `Auto` (serde alias or `legacy.rs`). Add a test.
2. **Agent forcing:** `providers/agent.rs:128` becomes `forced = request.web_search == Deep`. This
   reuses the forced first-round search (`agent/rounds.rs:36`) and the pre-search for models without
   tool calling. Remove `ChatRequest.web_toggle` and its frontend counterpart.
3. **Frontend:**
   - `types.ts`: `WebSearchMode = "off" | "auto" | "deep"`, `WebOverride = "off" | "auto" | "deep"`.
   - `lib/webSearch.ts::effectiveWebMode`: tool calling off → `"off"`; else override ?? configured
     ?? `"auto"`.
   - `chatStore.svelte.ts::cycleWeb` (Ctrl+W): off → auto → deep → off, starting from the effective
     mode.
4. **Editors:** `ModelEditor.svelte:26` and `ProfileEditor.svelte:148` options become off / auto /
   deep. Profiles may default to Deep.
5. **Footer** (`Footer.svelte:58-80,151-158,223-235`):
   - `data-state` ∈ off/auto/deep;
   - **only Deep is highlighted**, reusing the current `on` style. Off and Auto look as they do now;
   - the tooltip names the mode.

### Single-loop Deep (no subagents)
6. **Budgets:**
   - `config/settings.rs` `Advanced.deep: DeepSettings { max_rounds: 15, max_searches: 30,
     max_fetches: 40, max_fetched_chars: 400_000, max_turn_secs: 900, subagent_model: String /* "" */ }`;
   - defaults in `defaults.rs`;
   - expose in the Advanced settings UI.
7. **Budget:** `agent/budget.rs`: add `Budget::deep(&Advanced)` with its own clamps. Today's caps
   are rounds 1–10 and searches 1–20; deep allows e.g. rounds ≤ 30, searches ≤ 60. In `agent.rs`,
   choose budget, `Guidance` and `max_turn_secs` by mode (the turn clamp is `30..=3600`).
8. **Prompt:** `providers/prompt.rs`: `DEEP_GUIDANCE`, used instead of `TOOL_GUIDANCE` in Deep:
   - plan 3–6 sub-questions;
   - search and fetch per sub-question, official sources first;
   - keep a list of what's still uncovered;
   - stop when everything is covered or the budget is out.
9. **Final pass:** `agent/final_pass.rs`. In Deep, request a structured report: summary, findings
   per sub-question, open questions, all with `[n]` citations.
10. **Timeouts:** check that the stream inactivity timeout (`providers/stream.rs:137-205`) keeps
    resetting through long tool phases, and that Stop cancels a Deep turn cleanly.
11. **Docs:** `docs/agent.md` (modes, Deep flow) and `docs/limits.md` (Deep budgets).

### Paid-search warning toast
12. **Preflight (frontend only):**
    - **When:** switching to Deep, or sending in Deep, with a non-Antigravity model.
    - **Condition:** SearXNG has no base URL, or isn't first in `search.providers`.
    - **Action:** `notify(…, "warning")`:
      > "Deep research will use paid search (<providers>) — set up SearXNG in Settings → Web search."
13. **Runtime (backend):**
    - **When:** in a Deep turn, `search::run` returns results from a provider other than
      `searxng`.
    - **Action:** emit a new `search://paid` event **once per turn** with the provider and the
      reason SearXNG was skipped. Pass the deep flag via `ToolContext`.
    - **Frontend:** subscribe in `App.svelte` like `onSearchUnavailable` (`App.svelte:176`) and
      `notify(message, "warning")`.
14. Tests:
    - `"always"` → `Auto`;
    - the `cycleWeb` order;
    - `effectiveWebMode`;
    - Deep picks the deep budget and forced search;
    - `DEEP_GUIDANCE` only in Deep;
    - the paid-provider detection helper.

## Step 9 — Deep research with Antigravity (agy only)

Context: `docs/antigravity.md`. `agy` exposes `search_web`, `read_url_content`, `invoke_subagent`
and `define_subagent` (see `providers/antigravity/fixtures/tool.ndjson` init tools).

1. **Live probe first — this gates the step.** Run `agy` exactly as SlopChat does: `--sandbox`,
   stream-json in/out, empty workspace. Use a prompt that requires web search and a subagent.
   - Confirm `search_web`, `read_url_content` and `invoke_subagent` execute headless under
     `permission_mode: request-review` and are not auto-denied like `run_command`.
   - Save the output as a fixture (`fixtures/deep.ndjson`).
   - **If they are denied, stop and report back.** Do not work around it with
     `--dangerously-skip-permissions`.
2. **Footer:** for `antigravity/*` models the web button is currently disabled ("Antigravity uses its
   own tools", `Footer.svelte:58-70`). Make it a two-state toggle, normal ↔ deep. Off/Auto both mean
   "normal" for agy.
3. **Request:** pass the mode through to `antigravity::run` (`providers/antigravity/mod.rs`). In Deep,
   prepend agy-specific deep instructions to **this turn's prompt**, for both `Plan::Fresh` and
   `Plan::Resume`, because a resumed agy conversation never re-reads the system prompt.
   - Instructions: plan sub-questions, use `search_web`/`read_url_content`, use subagents for
     parallel sub-questions, finish with a report and a `Sources:` list of URLs.
4. **Sources:** parse the trailing `Sources:` list (markdown links or bare URLs) into `Source`s, so
   citation chips work. Keep it in a small pure function with tests.
5. **Timeout:** `--print-timeout` = `advanced.deep.max_turn_secs` in Deep.
6. **No SlopChat search:** no SlopChat search providers, budgets, subagents or paid-search warnings
   for agy (both 8.12 and 8.13 must skip agy).
7. **Tests:**
   - the deep prompt is prepended on both fresh and resume;
   - the session fingerprint still matches on the next turn: fingerprint the user's text, not the
     prepended instructions;
   - parsing the Sources list.
8. **Docs:** add a Deep section to `docs/antigravity.md` and update the "Requests SlopChat sends that
   `agy` ignores" list.

## Step 10 — Cheap subagents for Deep (non-agy models)

Active only when `advanced.deep.subagent_model` is non-empty. Otherwise Step 8's single loop is used.

**Why:** fetched pages are most of the tokens, so a cheap model reads them. Sub-questions run in
parallel, and the main model's context stays small.

1. **Prerequisite refactor:** check whether `agent/rounds.rs`/`agent/turn.rs` are tied to
   `tauri::ipc::Channel<StreamEvent>`. Introduce a minimal event-sink abstraction (trait or closure)
   so the round loop can run headless. Keep it minimal; it exists because there are now two callers.
2. **`agent/research.rs`:** a `research(question: string)` tool schema and executor, offered to the
   main model **only** in Deep with a subagent model. In that mode the main model gets `research`
   instead of `web_search`/`web_fetch`.
3. **`agent/subagent.rs`:** runs one headless turn of the round loop with the subagent model.
   - **Tools:** `web_search` + `web_fetch`.
   - **Budget:** small per-subagent (4 rounds, 6 searches, 8 fetches) **plus** a shared turn budget
     (`Arc<Mutex<Budget>>`) capped by the Deep limits.
   - **Sources:** a **shared** source counter (`Arc<AtomicU32>`), so `[n]` numbers are unique
     across subagents.
   - **Prompt:** "answer this sub-question, return ≤ ~1.5k tokens of notes with [n] citations".
   - **Returns:** notes + sources + usage.
   - If `subagent_model` is `antigravity/*`, run it via `antigravity::exec` (it takes no Channel)
     with a research prompt and parse sources as in Step 9.4.
4. **Orchestration:**
   - The main model plans sub-questions and calls `research` (parallel tool calls are already
     supported).
   - Execute at most **3 concurrently** (SearXNG rate limits).
   - Merge sources into the turn's sources; the main model may call again for gaps; the final pass
     writes the report (Step 8.9).
5. **Progress:** forward subagent steps to the UI labelled `sub-question i/n`, reusing
   `MessageSteps.svelte` step events.
6. **Usage:** add subagent tokens and cost to the message usage.
7. **Fallback:** if the subagent model can't call tools (`capabilities::supports`), fall back to the
   single loop and `notify` a warning once.
8. **Tests:**
   - the shared budget refuses calls once exhausted;
   - source numbers don't collide across concurrent subagents;
   - an empty `subagent_model` → single loop;
   - the concurrency limit is respected.
9. **Docs:** a subagent section in `docs/agent.md` with a flow diagram.
