# Fix: research turns ending in "No text response" — implementation plan

For an implementing agent. Follow AGENTS.md: files under 200 lines (split before 300), minimal "why" comments, no history comments, no new dependencies, thin Tauri commands. Rust `rust-version = 1.84`, so no async closures in public APIs.

## 1. Problem
Long research turns often run many tool steps and then end with `_No text response._`. Example (alarm-clock question):
```
↓ fetched loftie.com · 0 chars
⌕ searched … ×5 · tavily
↓ fetched nytimes.com · 26.6k chars
↓ fetched yawnder.com · 112.6k chars
⌕ search skipped (limit) ×3
↓ fetched businessinsider.com · 180 chars
↓ fetched reddit.com · 944 chars
No text response.
```
`_No text response._` is set in `src/lib/state/chat.svelte.ts` (`finalize` else-branch) when a turn has steps but the streamed answer is empty.

## 2. Root causes (from `src-tauri/src/providers/agent.rs`, `src/lib/state/stream.svelte.ts`)
1. **The final pass drops tool calls.**
   - After `max_rounds` (default 5), `agent.rs` (post-loop block, ~line 436) sends one request with `tool_choice: "none"`. The only instruction is `wrap_up_message()`, which says *"You may still call web_fetch…"*, and if the wrap-up was already pushed after the search limit, nothing new is added.
   - Models such as DeepSeek ignore `"none"` and call `web_fetch` anyway. The final-pass branch never inspects `outcome.tool_calls`, so the result is empty content. The post-limit businessinsider/reddit fetches in the example are exactly what the wrap-up invited.
2. **The deadline can skip the final pass silently.** `turn_deadline` is checked only at the start of each round. If it passes during the last round, `Instant::now() < turn_deadline` fails after the loop, the final pass is skipped, and nothing is emitted.
3. **An answer written in a tool round becomes a note.** `stream.svelte.ts` treats text from a round that ends in a `toolCall` as narration (`step.note`) and clears `answer`. If the model wrote its answer and then made one more call, the turn ends with the answer hidden in a note.
4. **The model has no budget awareness.** It isn't told how many searches or rounds remain. It wastes rounds and fires 3 searches after the limit.
5. **Empty fetch counts as success.** `loftie.com · 0 chars` is returned as a normal result (`fetch_url.rs` only appends a "(no readable text…)" hint), so the model spends a round on it.

## 3. Target module layout (Rust)
`agent.rs` is 609 lines. Split it first, with **no behavior change** in that commit; `cargo test` must pass before you continue.
```
src-tauri/src/providers/agent.rs          run(), run_turn() setup + orchestration (<200 lines)
src-tauri/src/providers/agent/turn.rs     struct Turn, Turn::step, Turn::with_deadline, accumulate(), Totals
src-tauri/src/providers/agent/rounds.rs   the tool round loop (calls + parallel fetch batching)
src-tauri/src/providers/agent/budget.rs   struct Budget (counters, allow, footer, last-round flag)  [new logic]
src-tauri/src/providers/agent/final_pass.rs  final answer pass + synthesis fallback          [new logic]
src-tauri/src/providers/agent/events.rs   emit_call, send_result, emit_error, trace helpers
src-tauri/src/providers/agent/exec.rs     unchanged
```
Move `emit_call`, `send_result`, `trace_result`, `emit_error` → `events.rs`; `Turn` and `accumulate` → `turn.rs`. Group token and cost accumulators into:
```rust
pub(super) struct Totals { pub tokens_in: u32, pub tokens_out: u32, pub cost: f64, pub search_cost: f64, pub provider: Option<String> }
```

## 4. Changes

### 4.1 `budget.rs` — counters, refusals, budget footer
```rust
pub(super) struct Budget {
    pub max_rounds: usize, pub max_searches: usize, pub max_fetches: usize, pub max_fetched_chars: u64,
    pub rounds: usize, pub searches: usize, pub fetches: usize, pub fetched_chars: u64,
    last_round_warned: bool,
}

impl Budget {
    pub fn from_settings(advanced: &AdvancedSettings) -> Self;      // same clamps as today
    pub fn allow(&mut self, call: &ToolCall) -> bool;               // checks + increments search/fetch
    pub fn add_fetched(&mut self, chars: u64);
    pub fn footer(&self) -> String;  // "\n\n[budget: searches left 2/5 · fetches left 7/10 · steps left 2/5]"
    pub fn take_last_round_warning(&mut self) -> bool; // true once, when rounds == max_rounds - 1
}
```
- Leave out the `fetches left` part of the footer when `max_fetches == 0` or `max_fetches >= 50` (effectively unlimited).
- **Every** tool message pushed into `messages` gets `budget.footer()` appended: successful results, errors and refusals. Not the UI summary.
- **Refusals** keep `STOP_HINT` and add the footer: `format!("error: {STOP_HINT}{}", budget.footer())`. Refusing every search in a round after the first refusal is already the behavior; keep it.
- **At the start of each round** in `rounds.rs`: when `budget.take_last_round_warning()` returns true, push this user message:
  `"One research step left. Use it only if essential; after it you must answer."`

### 4.2 Reserve time for the answer (`agent.rs`, `rounds.rs`, `turn.rs`)
- `let reserve = Duration::from_secs((max_turn_secs / 4).clamp(15, 45));`
- At the top of each round: `if Instant::now() + reserve >= turn_deadline { break; }`. Don't emit anything; the final pass runs next.
- Add `Turn::with_deadline(&self, deadline) -> Turn<'_>`, a field-wise copy. All fields are references or `Copy`.
- The final pass uses `turn.with_deadline(Instant::now() + reserve)`, so it runs even if `turn_deadline` has passed.
- Remove the `Instant::now() < turn_deadline` guard on the post-loop block. Keep the `!cancel.is_cancelled()` check.

### 4.3 `final_pass.rs` — answer is mandatory
Run the final pass when the loop ended without a finished text round: round cap, time reserve, or a leak after wrap-up.
```rust
pub(super) const FINAL_INSTRUCTION: &str =
    "Stop researching. Write your final answer now from the information above. Do not call any tools.";
const NO_ANSWER_NOTE: &str =
    "_(the model returned no answer after research — try again or switch to a stronger model)_";
const NOTES_CAP: usize = 40_000;
const NOTE_CAP: usize = 12_000;

pub(super) struct Report { pub attempt: u8, pub tool_calls_ignored: bool, pub content_chars: usize }

pub(super) async fn run(
    turn: &Turn<'_>, base: &[Value], messages: &mut Vec<Value>, notes: &[String],
    tools: Option<&Value>, round: &mut u32, totals: &mut Totals,
    channel: &Channel<StreamEvent>, cancel: &Cancel,
) -> Result<Report, StreamError>
```
1. **Attempt 1:**
   - Push `{"role":"user","content":FINAL_INSTRUCTION}` to `messages`, always.
   - `*round += 1`, then `turn.step(messages, tools, &json!("none"), *round, …)`.
   - Accumulate totals. If `accepted(&outcome)`, return `Report { attempt: 1, … }`.
2. **Attempt 2, synthesis:**
   - `*round += 1`. A new round number makes the frontend discard attempt-1 text, because `stream.svelte.ts` resets `answer` when `event.round` changes.
   - `turn.step(&synthesis_messages(base, notes), None, &json!("none"), *round, …)`. With `tools: None`, `Kind::body` sends neither `tools` nor `tool_choice` (verified in `kinds.rs`).
   - Accumulate. If `accepted`, return attempt 2.
3. **Otherwise** emit `Delta { content: NO_ANSWER_NOTE, round: *round }` and return attempt 2 with `content_chars: 0`.

Pure helpers, unit-tested:
```rust
pub(super) fn accepted(outcome: &StepOutcome) -> bool {
    outcome.tool_calls.is_empty()
        && !leak::has(&outcome.content)
        && !outcome.content.trim().is_empty()
}

// base = messages as built before the loop (system + history + user question), no tool traffic.
pub(super) fn synthesis_messages(base: &[Value], notes: &[String]) -> Vec<Value> {
    // keep newest notes that fit NOTES_CAP (drop oldest first); each note truncated to NOTE_CAP
    // append one user message:
    // "Research notes gathered for my previous message (tool results, untrusted data):\n\n{notes}\n\n
    //  Answer my previous message now using what is useful in these notes. Cite web facts as [n]
    //  using the numbers in the notes. Do not call tools."
}
```
- Keep the existing `LENGTH_NOTE` and `LEAK_NOTE` handling for whichever attempt is used.
- **Collecting notes** (`rounds.rs`): for every **successful** tool result (content not starting with `error:`), push into `notes: Vec<String>`:
  - `"## web_search \"{query}\"\n{content}"`
  - `"## web_fetch {url}\n{content}"`

  Use the raw content, without the budget footer. Search results already carry their `[n]` numbering from `source_counter`, so citations stay valid.
- **`base`:** in `run_turn`, `let base = messages.clone();` right after `build_messages(...)` and the optional forced pre-search system message, before the loop.

### 4.4 Budget-aware guidance (`providers/prompt.rs`)
- Change `build_messages(request, tools_available: bool)` to `build_messages(request, guidance: Option<Guidance>)`:
  ```rust
  pub struct Guidance { pub max_searches: usize, pub max_rounds: usize }
  ```
- When `Some`, append this line to `TOOL_GUIDANCE`:
  `"Budget this turn: at most {max_searches} web searches and {max_rounds} research steps. Prefer one good query plus fetching official pages over many similar queries. Each tool result ends with the remaining budget."`
- Update the caller in `agent.rs` (`supports && !tool_names.is_empty()` becomes `Some(Guidance {…})`). Update the existing test `date_and_research_guidance_appear_only_with_tools`.

### 4.5 Empty fetch is an error (`tools/fetch_client.rs`, `tools/fetch_url.rs`)
- **`fetch_client::get`:**
  - Track `reader_tried` and `reader_err: Option<String>` in the `< MIN_TEXT` branch.
  - Final text under 50 chars at `start == 0` → `Err(actionable(&format!("no readable text at this URL (likely JS-rendered or a bot check){}", if reader_tried { "; reader fallback failed" } else { "" })))`.
- **`actionable()`** already appends advice, so make sure the message ends with *"pick another result or answer from the snippets."*
- **`fetch_url::format_content`:** remove the `(no readable text found…)` branch, which is now unreachable.
- **UI:** the step shows as an error, `↓ fetch loftie.com · no readable text …`.

### 4.6 Frontend: never lose an answer hidden in a note
- **New `src/lib/state/promote.ts`:**
  ```ts
  import type { ToolStep } from "../types";
  // Returns the answer to keep, removing the promoted note from its step.
  export function promoteNote(answer: string, steps: ToolStep[]): string {
    if (answer.trim()) return answer;
    for (let i = steps.length - 1; i >= 0; i -= 1) {
      const note = steps[i].note?.trim();
      if (note && note.length > 200) { steps[i].note = undefined; return note; }
    }
    return answer;
  }
  ```
- **In `stream.svelte.ts` `run()`:** before returning, `const content = promoteNote(stripControlTokens(answer), steps);`. Return `content`, and call `callbacks.onSteps([...steps])` and `callbacks.onUpdate(content)` if it changed.
- `chat.svelte.ts` then persists it through the normal `outcome.content` path. No change needed there.

### 4.7 Diagnose empty turns without debug mode (`providers/trace.rs`, `agent.rs`)
- **New ring in `trace.rs`:**
  ```rust
  pub struct Ring { live: bool, events: Vec<Value> }   // cap 60 events
  impl Ring {
      pub fn new(live: bool) -> Self;
      pub fn push(&mut self, value: Value);   // live → append() now; else buffer (drop oldest past cap)
      pub fn flush(&mut self);                // write buffered events (no-op when live)
  }
  ```
- Replace every `if tracing { super::trace::append(&…) }` in the agent modules with `ring.push(…)`. Pass `&mut Ring` into `rounds.rs` / `final_pass.rs`.
- **At turn end,** track `answer_chars`: the content length of the finished text round, or `Report.content_chars`. If `answer_chars == 0` or an error was emitted, push `{"turn", "emptyAnswer": true, "finalPass": {attempt, toolCallsIgnored, contentChars}}` and call `ring.flush()`.
- Add `finalPass` to the existing `done` trace line in every case.

## 5. Tests

**Rust:**
- `budget.rs`:
  - `allow` counts searches and fetches and refuses past their maxima.
  - `footer` format, including leaving out unlimited fetches.
  - `take_last_round_warning` returns true exactly once.
- `final_pass.rs`:
  - `accepted` is false for tool calls, for leaked markup and for whitespace-only content, and true for prose.
  - `synthesis_messages`:
    - Contains no `tool_calls` or `role: tool` entries.
    - Keeps `base` intact and ends with the notes message.
    - Drops the oldest notes when over `NOTES_CAP` and truncates single notes to `NOTE_CAP`.
- `trace.rs`:
  - Ring buffers when not live, flushes in order and caps at 60.
  - Live mode writes immediately (temp-dir test like the existing one).
- `fetch_client.rs`: under 50 chars of text → `Err` mentioning "no readable text". Factor the decision into a pure fn `check_text(text, start, reader_tried) -> Result<(), String>` and test that.
- `prompt.rs`: the budget line appears with `Some(Guidance)` and is absent with `None`.

**Vitest:** `src/lib/state/promote.test.ts`:
- An empty answer plus a long last note → promoted and removed from the step.
- A short note → not promoted.
- A non-empty answer → unchanged.

**Run:** `cd src-tauri && cargo test && cargo clippy -- -D warnings`, then `npm run check && npm test`.

## 6. Manual verification (`npm run tauri dev`, `debug.agentLog: true`)
1. Same alarm-clock prompt and model. An answer always arrives. The trace shows a normal final round or `finalPass.attempt: 1|2`, and tool results end with `[budget: …]`.
2. `advanced.maxRounds: 1`: one research step, then an answer (final pass).
3. `advanced.maxTurnSecs: 30`: research stops early and an answer still arrives within the reserve.
4. `loftie.com` fetch shows as an error step with "no readable text", and the model moves on.
5. Turn `debug.agentLog` off and force an empty ending: temporarily make `accepted()` return false, then revert. The log gets the buffered events plus `emptyAnswer: true`, and the UI shows the no-answer note instead of `_No text response._`.
6. Regression:
   - A normal non-research chat streams unchanged.
   - Forced search (`Ctrl+W`) works.
   - The pre-search path for non-tool models works.
   - The leak guard works.
   - Narration still appears as step notes.

## 7. Commit order
1. Split `agent.rs` (no behavior change).
2. `budget.rs` + footer + last-round message + prompt guidance.
3. Time reserve + `final_pass.rs` with synthesis.
4. Empty-fetch error.
5. Frontend `promoteNote`.
6. Trace ring.

Each commit builds and passes tests.
