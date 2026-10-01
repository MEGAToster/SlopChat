# Fix: raw DSML tool-call text leaking into replies (DeepSeek V4.1 Flash)

## Context
After a few successful searches, DeepSeek V4.1 Flash (via OpenRouter, `:floor`) printed its native tool-call markup as the answer (`<｜DSML｜calls> <｜DSML｜invoke name="web_fetch"> …`) instead of calling tools or answering. Two things happened:
- The model tried to call `web_fetch`, a tool the app doesn't define (ours is `fetch_url`), and `web_search` again after the app had removed it.
- The provider received no matching tool definitions, so it passed the model's raw tool syntax through as plain `content`.

## Root causes (in `src-tauri/src/providers/agent.rs`)
1. **Tool definitions disappear mid-turn.** Once `searches >= MAX_SEARCHES` (2), line ~175 removes `web_search` from `tool_names`, so the next request no longer defines it. The final pass after `MAX_ROUNDS` (line ~182) sends **no tools at all** (`None`) while the message history is full of `tool_calls` and `tool` results.
   - DeepSeek still wants to call tools, and with no definitions the provider can't parse the call, so the template tokens leak as text.
   - Rule: tool definitions must stay identical for the whole turn. Limit tool use by *result* (an error message) or by `tool_choice: "none"`, never by removing definitions.
2. **No explicit "stop and answer" instruction.** The final pass relies on `tool_choice: "none"`. Many models (DeepSeek especially) ignore it in their output and still emit tool markup.
3. **Routing with `:floor`.** OpenRouter's `:floor` picks the cheapest provider, and some providers don't support or parse tool calls for this model. Without `provider.require_parameters: true`, OpenRouter can route a request that has `tools` to a provider that ignores them, which produces the same raw-text symptom.
4. **Tool name mismatch.** DeepSeek and many other models are trained on `web_fetch` / `web_search`. Our `fetch_url` invites hallucinated `web_fetch` calls.

## Fix
1. **Keep tool definitions stable** (`agent.rs`):
   - Compute `tools_value` once before the loop and send it on every step, including the final pass.
   - Remove the `tool_names.retain(...)` line. Enforcement stays where it already is: calls over the limit get `"error: search limit reached for this turn — answer with the results you have"` (the current `allowed` branch).
   - The final pass sends `tools_value` with `tool_choice: "none"` instead of `None`.
2. **Explicit wrap-up message:** before the final pass, and whenever the search limit is hit, append a `user` message: *"Tool budget for this turn is used up. Answer now using the results above. Do not call tools."* This is a user message, not a system one, because some providers drop late system messages.
3. **Require tool-capable providers** (`providers/kinds.rs::body`): when `tools` is present and the kind is OpenRouter, add `"provider": { "require_parameters": true }`, so routing only uses providers that honor `tools` / `tool_choice`. This keeps `:floor` cheap among those.
4. **Rename `fetch_url` → `web_fetch`** (`tools/mod.rs`, `tools/fetch_url.rs` schema name, profile defaults in `config/defaults.rs`):
   - Map the legacy name `fetch_url` in profile `tools` lists onto `web_fetch`.
   - In `tools::run`, also accept `fetch_url` as an alias, so older chats and models still work.
5. **Leak guard** (new `providers/leak.rs`, ~60 lines, unit-tested):
   - Detect native tool markup in streamed `content`: `<｜DSML｜`, `<｜tool▁calls▁begin｜>`, `<｜tool▁call▁begin｜>`, `<tool_call>`, `[TOOL_CALLS]`.
   - While streaming (`stream.rs`), buffer text after a marker instead of forwarding it as `Delta`, so markup never reaches the UI.
   - If a step ends with leaked markup and no real `tool_calls`, drop the markup from the content and run **one** extra pass with tools still defined, `tool_choice: "none"` and the wrap-up message. If that also leaks, end with whatever prose came before the marker plus the note *"(model tried to call tools in an unsupported format)"*.
   - Don't try to parse DSML into real tool calls. The formats are model-specific and change often.
6. **Limits:** raise `MAX_SEARCHES` to 3 (the screenshot shows the model wanting a third, more specific query). Keep `MAX_ROUNDS` at 4. Count `web_fetch` separately with its own cap of 4, since it's free.

## Files
- `src-tauri/src/providers/agent.rs`: stable tools, wrap-up message, final pass with tools, leak retry, limits.
- `src-tauri/src/providers/kinds.rs`: `provider.require_parameters` for OpenRouter when tools are sent.
- `src-tauri/src/providers/stream.rs`: hold back deltas after a leak marker.
- `src-tauri/src/providers/leak.rs` (new): marker detection and stripping, plus tests.
- `src-tauri/src/tools/mod.rs`, `tools/fetch_url.rs`, `src-tauri/src/config/defaults.rs` (+ config fixup): `web_fetch` rename with the `fetch_url` alias.
- Frontend: step-line label for `web_fetch` (wherever `fetch_url` is matched in `Message.svelte` / stream state).

## Verification
- **Unit tests:**
  - `leak.rs`: detects each marker, including when it's split across two deltas. Strips the markup and keeps the prose before it.
  - Agent loop: after the search cap, the request body still contains the identical `tools` array.
  - `kinds.rs` body: OpenRouter + tools → `provider.require_parameters == true`.
  - Config: `tools: ["fetch_url"]` → `["web_fetch"]`.
- **Manual:** run `npm run tauri dev` with `deepseek-v4.1-flash:floor` and repeat the prompt "please fetch the latest information about anthropics models and pricing". Expect:
  - At most 3 searches, then optional `web_fetch` steps.
  - A prose answer with `[n]` citations.
  - No `DSML` text at any point, including mid-stream.
- **Regression:** a tool-capable non-DeepSeek model (e.g. a Claude or Gemini profile) still searches and answers normally. A `forced` search (`Ctrl+W`) still works.
