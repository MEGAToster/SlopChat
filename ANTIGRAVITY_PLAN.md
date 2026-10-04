# Antigravity CLI (`agy`) as a SlopChat provider — implementation plan

## 1. Context
The goal is to run Google's Antigravity CLI (`agy`, the successor to Gemini CLI since June 2026) from SlopChat. `agy` runs under the Google account the user has already signed into. SlopChat sends input to it programmatically and streams its output into the chat UI.

**Decisions made by the user (do not revisit):**
| Topic | Decision |
|---|---|
| Shape | A **provider**: model refs `antigravity/<slug>` usable in profiles, the palette and the model editor. |
| Future | Other models may later use `agy` as a **deep-research tool**, so the core runner must not depend on the chat channel or the store. |
| History | **Resume `agy`'s own conversation** per chat (`--conversation <id>`), and re-seed a fresh conversation when the history diverged. |
| Safety | **Chat-only and isolated:**<br>• an empty SlopChat-owned workspace dir;<br>• `--sandbox`;<br>• never `--dangerously-skip-permissions`;<br>• shell commands stay soft-denied (the headless default). |
| Sign-in | Out of scope in-app. The user signs in by running `agy` once in a terminal. |

## 2. Rules for the executing agent
1. Read `AGENTS.md`:
   - files under 200 lines (split before 300);
   - comments explain *why*, never history;
   - Tauri commands only validate and delegate;
   - platform code only in `panel/`/`platform/`;
   - no speculative abstractions.
2. **Dependencies:** no new crates or npm packages. Use `tokio::process`, `tokio::io`, `serde_json` and `std::hash`.
3. **Git:** no commits or any other write git actions.
4. **Installing `agy`:** it is **not installed** on the dev machine. Before Phase 0, ask the user to install it (`curl -fsSL https://antigravity.google/cli/install.sh | bash`) and sign in by running `agy` once. Never run the installer yourself.
5. **Field names:** the `agy` field names in §3 come from public docs. **Phase 0 confirms them against real output before any parser exists.** If reality differs, reality wins; update this plan's §3 and `docs/antigravity.md`.
6. **Checks:** after every phase, run the verification commands in §11 and fix failures before continuing.
7. **Final report:** a phase-by-phase list of done / deviated (why) / open.

## 3. What we know about `agy` — **confirmed against 1.2.16 in Phase 0**
Sources: `--help`, live probes on 1.2.16, plus the docs listed earlier. Where the docs and the binary disagreed, the binary won. Fixtures live in `src-tauri/src/providers/antigravity/fixtures/`.

**Install and auth**
- **Binary:** `~/.local/bin/agy` on macOS/Linux. A Finder-launched macOS app doesn't inherit the shell `PATH`, so this path must be checked explicitly.
- **Version:** `agy --version` → `1.2.16` (plain `major.minor.patch`, no `v`).
- **Credentials:**
  - Google sign-in tokens live in the OS keyring (Keychain / Secret Service); headless runs reuse them.
  - There is **no "auth status" command**. Auth failures surface as an error whose text includes `authentication required` / `not logged in` / `not signed in`.
- **Settings file:** `~/.gemini/antigravity-cli/settings.json`; it has `permissions.allow`. Do not modify it.

**Invocation (headless, streaming)**
```
agy --input-format stream-json --output-format stream-json \
    --model <slug> [--conversation <id>] --sandbox [--print-timeout <N>s]
stdin:  {"event":"user","message":{"content":"<prompt>"}}\n   then close stdin
```
- `--input-format stream-json` requires `--output-format stream-json`.
- **Closing stdin** ends the session after the current turn completes.
- **Content:** only text is allowed; `content` may be a string or `[{type:"text",text}]`.
- **`--print-timeout` default is `0s` (wait indefinitely), not 5m.** A non-zero value returns partial output when it expires.
- **`--effort` must NOT be combined with `--model`:** every model returned by `agy models` already encodes effort in its slug, so `--model X --effort Y` fails with `--model X conflicts with --effort=Y` (or `--effort is not supported for model "X"`). Phase 0 therefore **drops `--effort`**: the model slug carries the effort. (`--effort` only works on the default model, which we never use.)

**Output (NDJSON on stdout, one object per line)**
| `event` | Fields |
|---|---|
| `init` | top-level `conversation_id`; nested `init{cwd, tools[], permission_mode, model?}`. `model` appears only when `--model` is passed. |
| `step_update` | nested `step_update{conversation_id, step_index, state, step_type, …}`. `state` ∈ `ACTIVE`/`DONE`/`ERROR`. `step_type` ∈ `user_input`/`agent_response`/`tool`/`system_message`. Text: `text_delta` (one or more `ACTIVE`, then a final `DONE` that carries `usage`). Tools: `tool_name`, `tool_info{name, parameters, output?}` on success, `tool_info{…, error{type,message}}` on failure. `user_input`/`system_message` carry no text and are ignored. |
| `result` | nested `result{conversation_id, status, response, error?, duration_seconds, num_turns, usage, denied_actions?}` |

`result` payload:
```json
{"conversation_id":"…","status":"SUCCESS","response":"…","duration_seconds":7.16,"num_turns":1,
 "usage":{"input_tokens":10415,"output_tokens":657,"thinking_tokens":616,"cache_read_tokens":8113,"total_tokens":11072}}
```
- On error, `status` is `ERROR` and an `error` string is present; `response` is empty.
- **Exit codes observed:** 0 = success, 1 = error. (`status`/`error` is the reliable signal, not the code.)
- **No thinking step:** reasoning is not streamed. `thinking_tokens` only appears in usage.
- **Tool output is a summary** (e.g. `"2 lines, 17 bytes"`), never the full tool payload.

**Other commands**
- **Models:** `agy models` (there is **no `--output-format` flag** in 1.2.16). stdout is `slug<TAB>Display Name`, one per line; a progress line goes to stderr. Parse the tab-separated text, not JSON.
- **Version:** `agy --version`.

**Known bugs / behaviors to defend against**
- **#1065 empty success:** `SUCCESS` with an empty `response` and no streamed text. Treat it as an error.
- **Denied command looks like empty success:** when a tool needs a permission headless can't prompt for, `agy` auto-denies, exits 0 with `status:SUCCESS`, an empty `response`, and `result.denied_actions`. The stderr says `no output produced … auto-denied`. The empty-response guard covers it.
- **Unknown conversation is silent, not an error:** resuming an id `agy` doesn't know prints `warning: conversation "X" not found` to stderr and starts a **fresh** conversation with a **different** `conversation_id`, exit 0. Detect by comparing the returned `conversation_id` to the requested one (or the stderr warning) and re-seed once.
- **Unknown model:** exit 1, `status:ERROR`, `error` contains `is not recognized as a known model`.
- **Cancel (`SIGINT`):** exit 1, `status:ERROR`, `error` is `context canceled`; stderr `error: context canceled`. Treat as cancelled when our `Cancel` fired.
- **A different `--model` is accepted on resume**, so §6 must **not** require the stored model to match.

## 4. Architecture

```
Frontend send ─▶ commands::send ─▶ providers::client::stream ─▶ providers::agent::run
                                                                     │ model.provider == "antigravity"
                                                                     ▼
                                                     providers::antigravity::run(app, request, channel, cancel)
                                                       │ session::plan()  ◀── store: chats.agy_session
                                                       │ exec(opts, on_event)  ── spawns `agy` (process.rs)
                                                       │     stdout NDJSON ─▶ events::parse_line ─▶ AgyEvent
                                                       │     AgyEvent ─▶ events::Mapper ─▶ StreamEvent ─▶ Channel
                                                       └ on SUCCESS: store session; emit Usage + Done
```
- **No `Kind` variant:** `antigravity` is **not** added to `providers/kinds.rs::Kind`. `Kind` is built around HTTP endpoints and request bodies, and `agy` has neither. Dispatch happens by provider-name string before `Kind::resolve`.
- **Reuse, don't duplicate:**
  - `Cancel` (`providers/stream.rs`: `is_cancelled()`, `cancelled().await`);
  - `StreamEvent` and `ProviderModel` (`providers/mod.rs`);
  - `ChatRequest` / `ChatMessage` (`providers/mod.rs`);
  - `paths::data_dir()`, `paths::context_path()`;
  - `trace::Ring` (`providers/trace.rs`, which only writes when the agent log is on);
  - `Store::conn()` (`store/mod.rs`);
  - `Settings.advanced.{response_timeout_secs, max_turn_secs}` (`config/settings.rs`).

## 5. Module layout — `src-tauri/src/providers/antigravity/`
Target size is under 200 lines per file.

| File | Contents |
|---|---|
| `mod.rs` | `pub mod`s, `pub const PROVIDER: &str = "antigravity"`, and `pub async fn run(app, request, channel, cancel)`. `run` is the provider glue: request → `ExecOptions`, session handling, mapping, Usage/Done. |
| `exec.rs` | Channel-free core: `ExecOptions`, `AgyResult`, `AgyError`, `pub async fn exec(opts: ExecOptions, cancel: &Cancel, on_event: impl FnMut(&AgyEvent)) -> Result<AgyResult, AgyError>`. **This is the future research-tool entry point.** It must not touch the store, the `Channel` or `ChatRequest`. |
| `process.rs` | Spawning, stdin write and close, line reading, stderr capture, cancel and deadline handling. |
| `events.rs` | `AgyEvent` enum and the pure `parse_line(&str) -> Option<AgyEvent>`. |
| `mapper.rs` | `Mapper` turns `AgyEvent`s into `StreamEvent`s (round counting, tool ids). Pure, with no I/O. |
| `session.rs` | `Stored`, `Plan`, `pub fn plan(...)`, and the seed-prompt builder. Pure. |
| `binary.rs` | Finds the binary, checks the version, and provides `status()` for the settings UI. |
| `models.rs` | `pub async fn list() -> Vec<ProviderModel>`. |
| `fixtures/` | NDJSON/JSON captured in Phase 0 (tests only, `include_str!`). |

### 5.1 Types (sketch — adjust names to the confirmed shapes)
```rust
// events.rs
pub enum AgyEvent {
    Init,
    Text(String),                                   // step_update.text_delta
    Thinking,                                       // only if Phase 0 finds a thinking step
    Tool { key: Option<String>, name: String, params: Value, output: Option<String>, failed: bool },
    Result(AgyResult),
    Other,                                          // checkpoint, user_input, unknown → ignored
}
// exec.rs
pub struct ExecOptions { pub model: String, pub effort: Option<String>, pub conversation: Option<String>,
                         pub prompt: String, pub timeout: Duration }
pub struct AgyResult { pub conversation_id: String, pub status: String, pub response: String,
                       pub input_tokens: u32, pub output_tokens: u32, pub thinking_tokens: u32 }
pub enum AgyError {
    NotInstalled, TooOld(String), NotSignedIn,
    UnknownConversation,               // resume id rejected → caller retries fresh once
    UnknownModel(String),
    Cancelled, TimedOut,
    Failed(String),                    // last stderr line / status
}
```
- **Unknown fields:** `parse_line` ignores them and never errors on unknown `event` values (forward compatibility).
- **Bad lines:** a malformed line becomes `None`. It is logged to the trace and skipped, and is not fatal.

### 5.2 `binary.rs`
- **`locate() -> Option<PathBuf>`:**
  1. a test-only override (`#[cfg(test)] thread_local`/`static` set via `set_override_for_tests`);
  2. `dirs`: `$HOME/.local/bin/agy`;
  3. walk `std::env::var_os("PATH")` with `std::env::split_paths`, picking the first executable `agy`.

  No shell and no `which` crate.
- **`version(path) -> Result<Version, String>`:** run `agy --version` (2 s timeout) and parse the first `\d+\.\d+\.\d+`.
- **`check() -> Result<PathBuf, AgyError>`:**
  - a missing binary → `NotInstalled`;
  - a version below 1.1.15 → `TooOld(v)`;
  - cache a successful result in a `static OnceLock<Mutex<Option<(PathBuf, Version)>>>` and clear it when a spawn fails with NotFound.
- **`status() -> Status{ path: Option<String>, version: Option<String>, ok: bool, note: Option<String> }`** for the settings UI. `note` is "update recommended (≥ 1.2.16)" when below that.

### 5.3 `process.rs`
- **Build the command** with `tokio::process::Command::new(path)`:
  - `.current_dir(workspace())`;
  - `.args([...])` as in §3;
  - `.stdin(piped).stdout(piped).stderr(piped).kill_on_drop(true)`;
  - inherit the environment (needed for keyring/D-Bus access).
- **`workspace()`:** `paths::data_dir().join("antigravity").join("workspace")`, created with `create_dir_all` on each run. It is never a user dir.
- **Write the prompt:** `serde_json::to_string(&json!({"event":"user","message":{"content": prompt}}))` + `\n`, then `drop(stdin)` to close it.
- **Read stdout** with `BufReader::new(stdout).lines()` inside a `tokio::select!` loop:
  - a line → `parse_line` → `on_event`; keep the last `Result`;
  - `cancel.cancelled()` → `child.start_kill()`, then `wait` → `AgyError::Cancelled`;
  - `tokio::time::sleep_until(deadline)` → kill → `TimedOut`, where deadline = `timeout + 10s`. `agy`'s own `--print-timeout` should fire first and return partial output.
- **stderr:** read it in a spawned task into a `String` capped at 64 KB (keep the tail).
- **After EOF,** `child.wait()` gives the exit status. Classify (strings confirmed in Phase 0):
  - stderr contains `authentication required` → `NotSignedIn`;
  - an unknown-conversation message → `UnknownConversation`;
  - an unknown-model message → `UnknownModel`;
  - exit 0 with no `Result` → `Failed("agy ended without a result")`;
  - non-zero → `Failed(last non-empty stderr line)`.
- **Status:** `Result.status` other than `SUCCESS` → `Failed(status)`, except `CANCELED`/`INTERRUPTED` while `cancel.is_cancelled()` → `Cancelled`.

### 5.4 `mapper.rs` — `AgyEvent` → `StreamEvent`
State: `round: u32`, `text_in_round: bool`, `tool_seq: u32`, `open: HashMap<String, String>` (agy tool key → our id), `streamed_chars: usize`.

| AgyEvent | Emits |
|---|---|
| `Text(t)` | `Delta{content: t, round}`; set `text_in_round = true`; add to `streamed_chars` |
| `Thinking` | `Reasoning` |
| `Tool` (no output yet) | If `text_in_round`, then `round += 1` and reset it. Then `ToolCall{id: "agy-{seq}", name, args: params}` and remember it in `open` |
| `Tool` (with output) | If it isn't known yet, `ToolCall` first. Then `ToolResult{id, ok: !failed, skipped: false, summary: first line of output, max 120 chars, sources: vec![]}` |
| `Result` / `Init` / `Other` | nothing here (`run` handles `Result`) |

- **Rounds:** after a `ToolResult`, the next `Text` starts a new round (`round += 1` on the first text after a tool). This mirrors how `src/lib/state/stream.svelte.ts` treats each new `round` as a fresh answer buffer, so text written before tools becomes the step's `note`.
- **Phase 0 dependency:** if Phase 0 shows that `agy` sends one tool event per call with the output already included, the "no output yet" branch is unused. Keep it only if real data needs it.

### 5.5 `mod.rs::run` — provider glue
1. **Read the request:** `model = request.model["antigravity/".len()..]`, and `effort = request.reasoning` filtered to `low|medium|high`.
2. **Reject images:** if the newest user `ChatMessage.content` has any non-`text` part (an image), emit `Error("Antigravity accepts text only — remove image attachments")` and return. Text parts (inlined files) are joined with `\n\n`.
3. **Load the session:** `stored = chats::get_agy_session(&store.conn(), &request.chat_id)` (§6). Drop the lock right away; never hold the DB mutex across `.await`.
4. **Plan:** `plan = session::plan(&request, stored.as_ref(), &model)`, which gives `Resume{id, prompt}` or `Fresh{prompt}`.
5. **Run:** `exec(ExecOptions{…, conversation, prompt, timeout: Duration::from_secs(max_turn_secs)}, &cancel, |e| { ring.push(raw); for ev in mapper.map(e) { channel.send(ev) } })`.
6. **Handle the outcome:**
   - **`Err(UnknownConversation)` on a resume:** clear the stored session and retry **once** with `session::fresh_prompt(...)`.
   - **`Ok(result)` with `status == SUCCESS`:**
     - an empty `response` with `mapper.streamed_chars == 0` → emit `Error("Antigravity returned an empty response")` (#1065) and don't store the session;
     - a non-empty `response` with `streamed_chars == 0` → emit one `Delta{content: response, round}`;
     - store the session: `{id: result.conversation_id, model, users, hash}` (§6);
     - emit `Usage{tokens_in: input_tokens, tokens_out: output_tokens + thinking_tokens, cost: None, provider: Some("antigravity")}`, then `Done`.
   - **`Err(NotInstalled | TooOld | UnknownModel)`:** these must trigger the profile **fallback**. Return them to `agent::run` as `TurnError::Unavailable(msg)` (see §7.1) instead of emitting an error yourself.
   - **`Err(Cancelled)`:** emit nothing more, matching how HTTP providers end on cancel. Check `agent.rs` and mirror its behaviour, including whether `Done` is sent.
   - **Other errors:** emit `Error(user_message(err))`, then `Done` if the HTTP path does the same after errors (mirror it).
7. **User-facing messages** (`user_message`):
   - `NotSignedIn` → "Antigravity isn't signed in. Run `agy` once in a terminal and sign in.";
   - `TimedOut` → "Antigravity took longer than the turn limit (Advanced → max turn time).";
   - `Failed(s)` → "Antigravity: {s}".

### 5.6 `models.rs`
- **`list()`:**
  - if `binary::check()` fails → `vec![]`;
  - otherwise run `agy models --output-format json` with a 10 s timeout and parse the shape found in Phase 0;
  - map each entry to `ProviderModel{provider: "antigravity", id: slug, name: display name if present}`;
  - on any error → `vec![]`; never fail the whole listing.
- **No caching** beyond what `list_all` callers already do.

## 6. Conversation resume (per chat)
**Storage — migration V6** (`store/migrations.rs`, after `SCHEMA_V5`):
```sql
ALTER TABLE chats ADD COLUMN agy_session TEXT;
```
- **Store functions** in `store/chats.rs`, following the `set_model` style (`chats.rs:197`):
  - `pub fn get_agy_session(conn, id) -> Result<Option<String>, String>`;
  - `pub fn set_agy_session(conn, id, Option<&str>) -> Result<(), String>`.
- **Don't add it to the `Chat` struct** or `COLUMNS`: the frontend never needs it.
- **Value:** JSON `{"id":"<conversation_id>","model":"<slug>","users":<n>,"hash":"<16 hex>"}`.

**Fingerprint:** only **user** messages count. SlopChat post-processes assistant text before saving it (`stripControlTokens`, `promoteNote` in the frontend), so assistant text would never match.
```rust
fn user_texts(messages: &[ChatMessage]) -> Vec<String>   // role == "user", text parts joined
fn fingerprint(texts: &[String]) -> String               // std DefaultHasher over each text + a 0 separator, hex
```

**`session::plan(request, stored, model) -> Plan`.** With `users = user_texts(&request.messages)` (it includes the new message last):
- **`Resume{id, prompt: users.last()}`** when all hold:
  - `stored` is present;
  - `stored.model == model`. If Phase 0 shows `--conversation` accepts a different `--model`, drop this condition;
  - `users.len() == stored.users + 1`;
  - `fingerprint(&users[..stored.users]) == stored.hash`.
- **Otherwise `Fresh{prompt: fresh_prompt(request)}`.**

After success, the new stored value has `users = users.len()` and `hash = fingerprint(&users)`.

| Situation | Result |
|---|---|
| Normal follow-up | Resume |
| Edited / regenerated earlier message (rewind) | Fresh (hash mismatch, or count not +1) |
| Retry last message (same users, count not +1) | Fresh |
| Profile switched to OpenRouter for a turn, then back | Fresh (count +2) |
| Model slug changed | Fresh |
| Stored id unknown to `agy` | Fresh (retry once, §5.5) |

**`fresh_prompt(request)`** builds the seed. Plain text, because `agy` has no `--system` flag:
```
<system block: context.md if request.use_context, then request.system — same order as prompt.rs::build_messages>

Conversation so far:
User: …
Assistant: …
(all messages except the newest user message; text parts only; image parts replaced by "[image omitted]")

New message:
<newest user text>
```
- **Shared system builder:** extract the context/system part of `providers/prompt.rs::build_messages` into `pub fn system_text(request) -> String` and use it from both places. Don't copy it.
- **No SlopChat additions:** leave out the CITE/tool guidance (it's for SlopChat's tools, not `agy`'s).

## 7. Integration points (existing files)
### 7.1 `providers/agent.rs::run`
- **Dispatch:** right after `ModelRef::parse` succeeds and before the first `run_turn`, branch on `model.provider == antigravity::PROVIDER`.
- **Fallback path:** `run_turn` already returns `TurnError::Unavailable` for the HTTP path. Make the antigravity path return the same `Result<(), TurnError>`, so the existing fallback block (`agent.rs:46-70`) is reused for both. Either:
  - put the branch inside `run_turn` before `Kind::resolve` (`agent.rs:98`) — preferred, a smaller diff;
  - or factor a tiny helper.
- **Fallback chain:** the fallback model can itself be an HTTP model, and the reverse also works.

### 7.2 Keep HTTP-only lookups away from `antigravity`
- `providers/capabilities.rs` (tool support / reasoning info) and `providers/generation.rs` must short-circuit for provider `antigravity`:
  - tools: "unsupported";
  - reasoning: levels `low|medium|high`, default `medium`.
- Find every `Kind::resolve` / `ModelRef` consumer outside the agent (`grep -n "Kind::resolve\|ModelRef::parse" src-tauri/src`). Make each one handle `antigravity` gracefully, without a "provider not supported" error. Example: the `model_reasoning` command used by the footer.

### 7.3 `providers/mod.rs::list_all`
Add `antigravity::models::list()` as a fourth branch of the `tokio::join!`, then `extend`.

### 7.4 Commands, `main.rs`, `ipc.ts`
- **New command:** `antigravity_status` → `providers::antigravity::binary::status()`. It's `#[tauri::command(async)]` and only delegates.
- **Register** it in `main.rs`'s `generate_handler!`, and add `antigravityStatus()` plus the `AntigravityStatus` type in `src/lib/ipc.ts` / `src/lib/types.ts`.

### 7.5 `Cargo.toml`
- **tokio features:** add `"process", "io-util", "time"` (currently `macros, net, rt, sync`).
- **No new crates.**

### 7.6 Frontend
| File | Change |
|---|---|
| `src/components/MessageSteps.svelte` | It renders **every** non-`web_fetch` step as "searched “…”". Add a generic branch for other tool names: glyph `›`, then `{name} {short target}` while running, `{name} · {summary}` when done, and `{name} failed · {summary}` on error. Target = the first string value among `args.CommandLine`, `args.path`, `args.url`, `args.query`. Keep search/fetch rendering unchanged. |
| `src/lib/usageInfo.ts` | `providerLabel`: when `apiProvider === "antigravity"` and there's no `usage.provider`, return `"antigravity"`. The backend already sends `provider: "antigravity"`, so this only matters for history rows. Add a test in `usageInfo.test.ts`. |
| `src/components/Footer.svelte` | When the model's provider is `antigravity` (split with `splitModelRef`), show web search as not applicable: disabled, with the title "Antigravity uses its own tools". This follows the existing `toolCalling === false` handling (`Footer.svelte:57-66,148`). Keep the effort button. |
| `src/components/ModelEditor.svelte` | Make sure saving `antigravity/<slug>` needs no API key and offers no key field, the way `ollama` is treated. If the editor offers provider-specific fields (temperature, tools, webSearch), hide or disable the ones `agy` ignores: temperature, tools, toolCalling, webSearch. |
| `src/components/Settings.svelte` (or a new `AntigravitySection.svelte`, since Settings is large) | A read-only **Antigravity** block:<br>• binary path + version, or "Not installed — `curl -fsSL https://antigravity.google/cli/install.sh \| bash`";<br>• the line "Sign in: run `agy` once in a terminal.";<br>• the `note` from `status()`;<br>• a **Refresh** button that calls `antigravityStatus()` again.<br>Load it lazily when Settings opens. |
| `src/lib/state/paletteItems.ts` / palette "add" mode | Provider models already list everything from `list_all`. Check that `antigravity/*` entries appear and that "add" creates a correct `ModelConfig{model: "antigravity/<slug>"}`. |

## 8. Requests SlopChat sends that `agy` ignores
Write this list in `docs/antigravity.md` so users aren't surprised:
- **Ignored fields:** `temperature`, `tools`, `toolCalling`, `webSearch`/`webToggle`, `maxCompletionTokens`, the frequency/presence penalties, and SlopChat's search/fetch budgets. `agy` uses its own tools and limits.
- **`reasoning`:** mapped to `--effort` when it is `low`, `medium` or `high`; otherwise omitted.
- **Timeout:** `advanced.max_turn_secs` becomes `--print-timeout`, plus the hard kill 10 s later.
- **Cost:** `cost` is always `None`. The sign-in plan isn't billed per token, and `agy` reports no cost.

## 9. Phases (do in order)
### Phase 0 — Capture real `agy` behaviour (needs the user's install + sign-in)
Work in a temp dir (`mktemp -d`) and save sanitized output in `src-tauri/src/providers/antigravity/fixtures/`. Remove account emails and absolute home paths.

| # | Command | Save as / answer |
|---|---|---|
| 1 | `agy --version` | Record the version in `docs/antigravity.md` |
| 2 | `agy models --output-format json` | `models.json`; record the shape |
| 3 | `printf '%s\n' '{"event":"user","message":{"content":"Reply with exactly: hello there"}}' \| agy --input-format stream-json --output-format stream-json --sandbox` | `text.ndjson`. Confirm `text_delta`, the `result` nesting, and `conversation_id` |
| 4 | Same, with the prompt "List the files in the current directory, then say done" (create 2 dummy files first) | `tool.ndjson`. Does a tool arrive as one event or start + end? Is there a step key? Is `output` present? Is there a failed flag? |
| 5 | Repeat #3 with `--conversation <id from #3>` and the prompt "What did I ask you before?" | `resume.ndjson`. Confirm context continuity |
| 6 | #5 again with a different `--model` | Is it accepted? Decides the `model` condition in §6 |
| 7 | #3 with `--effort high` on a thinking model | Is there a thinking step type? `thinking.ndjson` |
| 8 | `--conversation does-not-exist` | stderr text + exit code → `UnknownConversation` classifier |
| 9 | `--model nope` | stderr + exit → `UnknownModel` classifier |
| 10 | Start #4 and `kill -INT` it after 2 s | Output tail + exit → cancel behaviour |
| 11 | Ask the user to run `/logout` in `agy`, re-run #3, then sign in again | stderr text for `NotSignedIn` |
| 12 | Hand-write `empty_success.ndjson`: init, one step, `result` with `SUCCESS` + `""` | Covers #1065 |

Update §3 of this plan and draft `docs/antigravity.md` with the confirmed shapes.

### Phase 1 — Pure parsing and mapping
- `events.rs`, `mapper.rs`, and fixture tests: each fixture parses, the mapper output is snapshot-compared as a `Vec<StreamEvent>`, and malformed/unknown lines are skipped.
- No process code yet.

### Phase 2 — Binary + process + exec
- `binary.rs`, `process.rs`, `exec.rs`, and the tokio features.
- **Fake-binary tests** (`#[cfg(unix)]`) using a shell script written to a temp dir and set through `binary::set_override_for_tests`:
  - `cat` a fixture → full event sequence plus `AgyResult`;
  - print 1 line, then `sleep 30` → cancel after 100 ms gives `Cancelled`, and the child is gone (`kill -0` fails);
  - `sleep 30` with timeout 1 s (deadline shortened in the test) → `TimedOut`;
  - `echo "authentication required" >&2; exit 1` → `NotSignedIn`;
  - the script reads stdin and echoes it to a file → assert that one valid JSON line was written and stdin was closed (the script sees EOF).

### Phase 3 — Session + store
- Migration V6 and `get/set_agy_session` with tests:
  - migrate an existing V5 DB;
  - round trip;
  - `None` clears.
- `session.rs` with table-driven tests for every row of the §6 table, plus a `fresh_prompt` formatting test.
- Extract `prompt.rs::system_text` and make sure the existing prompt tests still pass.

### Phase 4 — Provider glue
- `mod.rs::run`, the dispatch in `agent.rs`, short-circuits in `capabilities.rs` / `generation.rs` / other `Kind::resolve` users, and `list_all`.
- **Test:** a request whose model is `antigravity/x` with the binary override set to the fixture script → channel events in the right order, the session stored, Usage and Done emitted. Use whatever test pattern exists for `agent::run`; if none can host a `Channel`, test `run`'s inner function that takes an `FnMut(StreamEvent)` sink instead.

### Phase 5 — Frontend
- Everything in §7.6, plus the `antigravity_status` command and IPC.
- Tests: `usageInfo` and the `MessageSteps` generic branch (if it can be tested with the existing vitest setup), plus `npm run check`.

### Phase 6 — Docs
- **New `docs/antigravity.md`:**
  - overview + ASCII diagram (§4), with mermaid in a `<details>` block following `docs/README.md` conventions;
  - the confirmed event shapes;
  - the resume rule (§6);
  - isolation/permissions;
  - the ignored-fields list (§8);
  - known `agy` bugs + defenses;
  - troubleshooting: not installed, not signed in, too old, timeout.
- **`docs/README.md`:** add a `[[antigravity]]` row.
- **`README.md`:** Providers bullet → "OpenRouter, Ollama, Gemini API, Antigravity CLI (uses your signed-in Google account; install `agy` and run it once to sign in)".
- **`docs/limits.md`:** note that `max_turn_secs` maps to `--print-timeout` for Antigravity.

## 10. Future: deep-research tool (not in this plan; keep possible)
- **Already in place:** `antigravity::exec(ExecOptions, &Cancel, on_event)` returns `AgyResult` with no Channel, store or ChatRequest dependency.
- **A later `tools/research.rs` can:**
  - call `exec` with `conversation: None`, a research prompt and a fresh temp workspace;
  - turn `Tool` events into a summary;
  - return `response` as the tool output.
- **Don't add the tool, its schema, or settings now.**

## 11. Verification
After each phase:
```sh
cd src-tauri && cargo test && cargo clippy -- -D warnings
cd .. && npm run check && npm test
```
Manual end-to-end (`npm run tauri dev`, `agy` signed in):
1. **Add the model:** palette → add → provider models → pick an `antigravity/…` model and save. It appears in the palette and footer.
2. **Stream:** send "hi". Text streams, the footer shows usage with the `antigravity` label, and there is no cost.
3. **Resume:** ask a follow-up that depends on the previous answer. It answers correctly, and with **Advanced → Agent log** on, the trace shows `--conversation` used (a resume) without a re-seed.
4. **Edit:** edit the first message and resend. The trace shows a fresh conversation, and the answer reflects the edit.
5. **Tools:** ask "list files in the current directory". A generic tool step renders, and the listing is of the empty SlopChat workspace (`~/.local/share/SlopChat/antigravity/workspace`), never the user's home or project.
6. **Shell denied:** ask it to run `touch ~/x`. It is denied or soft-denied and `~/x` does not exist.
7. **Stop:** stop mid-answer. The UI stops and `pgrep -fl agy` shows no leftover process.
8. **Images:** attach an image. There's a clear "text only" error and nothing is sent.
9. **Signed out:** `/logout` in `agy`, then send → "not signed in" message. Sign in again.
10. **Missing binary:** rename `~/.local/bin/agy` → the profile's fallback model answers. Without a fallback there's a clear "not installed" error, and Settings shows "Not installed". Rename it back and press **Refresh** → the version shows.
11. **Model switch:** in the same chat, switch to an OpenRouter model for one turn, then back to Antigravity. That turn is re-seeded (Fresh) and still has the full context.
