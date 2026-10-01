# SlopChat — Tuning the web research agent (implementation plan)

Follow CLAUDE.md: small files (<200 lines), minimal "why" comments, no history comments, no new crates, thin commands, secrets only in the keychain.

## Context
Prompt: "please fetch the latest information about anthropics models and pricing". With DeepSeek V4.1 Flash, the visible reply was:

> I'll fetch the latest Anthropic model and pricing information for you.Let me pull the authoritative details from Anthropic's official docs.Let me get the official pricing page and models overview for the details.I wasn't able to open the individual pages (the fetch tool isn't available in this turn), so the following is compiled from the search results…

The model can't be fine-tuned, but the agent around it can. The problems are ours, in four layers: **fetch reliability, tool feedback, prompting, and output handling.** There's also no visibility into what actually happened.

## Diagnosis (current code)
1. **`web_fetch` probably failed silently** (`src-tauri/src/tools/fetch_url.rs`):
   - **No `User-Agent` header.** reqwest sends none, and sites behind Cloudflare or bot protection (anthropic.com, platform.claude.com) answer 403.
   - **Pages over 2 MB are rejected outright** instead of truncated. Next.js docs pages are often larger than that.
   - **Errors go back as `error: fetch failed: 403 Forbidden`.** The model reads that as "the tool isn't available" and gives up, instead of trying another URL.
   - **Redirects aren't re-checked by the SSRF guard** (the guard only checks the first host), and DNS is resolved twice (guard vs. reqwest). This is a security gap.
2. **Tables are flattened** (`tools/html.rs`): `td`/`th` aren't separated, so a pricing table becomes `ModelInputOutputOpus 5.5$5$25…`, which is useless for the exact question asked. There's also no `<main>`/`<article>` preference and no `<title>`.
3. **Prompting** (`src-tauri/src/providers/prompt.rs`):
   - **No current date in the system prompt.** "Latest" questions need it, and the model guesses the year in its queries.
   - **No research strategy:** search → fetch 1–3 official pages → answer.
   - **No rule against narration.** Nothing says not to narrate tool use or claim a tool is unavailable unless a tool result said so.
   - **Tool descriptions are one-liners** (`"Fetch a web page and return its text"`).
4. **Narration is glued into the answer.** Text the model writes before each tool call ("Let me pull…") streams as `delta` into the same message body, with no separator between rounds (`src/lib/state/stream.svelte.ts`). The result is `for you.Let me pull…`.
5. **`MAX_ROUNDS = 4`** (`providers/agent.rs`) is tight when fetches run one per round: search, fetch, fetch, and the round limit is reached.
6. **No trace.** Failed or limit-refused calls are partly hidden from the UI. You can't see which URLs were fetched, what the result was, or why.

## Fix

### 1. Observability first
- **Agent trace log** (new `src-tauri/src/providers/trace.rs`):
  - When `settings.debug.agentLog` is true (default false), append one JSON line per event to `~/.local/share/SlopChat/logs/agent.jsonl`.
  - Each line holds: turn id, round, model, provider, `tool_choice`, each tool call (name, args), the result (ok, error text, byte length, first 300 chars), finish reason, and usage.
  - Rotate the file at 5 MB.
  - Add a palette action "Open agent log".
- **Show every tool step in the UI**, including failures, with the error text (`⌕ fetch platform.claude.com/… · 403 blocked`). Limit-refused calls show as `· skipped (limit)`.

### 2. Make `web_fetch` reliable (`tools/fetch_url.rs`)
- **Browser-like request headers:** a desktop Safari `User-Agent`, `Accept: text/html,application/xhtml+xml`, `Accept-Language: en`.
- **Truncate instead of failing:** stream the body with `response.chunk()` and stop at 2 MB.
- **Redirects:** build a dedicated client with `redirect::Policy::custom` that runs `guard()` on every hop (max 5). Connect to the IP the guard resolved (`ClientBuilder::resolve(host, addr)`), so DNS can't change between the check and the request.
- **Parameters:** add optional `start` (char offset, for paging long pages) and `maxChars` (default 12,000, cap 20,000) to the schema.
- **Result header:** the result starts with `Title: … \nURL: … (chars X–Y of Z)`, so the model knows when there's more to read.
- **Fallback for blocked or JS-only pages:** on 403/429/503, or when extracted text is under 500 chars, retry once through **Jina Reader**, `GET https://r.jina.ai/<url>`. It's free without a key (rate-limited) and returns markdown of the rendered page.
  - Setting `search.readerFallback: true` (default true). Mark the summary `· via reader`.
- **Errors the model can act on:** return `error: <reason>. Try another URL from the search results or answer from the snippets.` Never return a bare status code.

### 3. Better text extraction (`tools/html.rs`)
- Prefer the content of `<main>` or `<article>` when present. Fall back to `<body>`.
- Separate table cells with ` | ` and end rows with a newline, so tables come out as pipe-delimited rows. Add a small fixture test with a pricing table.
- Extract `<title>` for the result header.
- Add `aside`, `form`, `button` and `[aria-hidden]`-style chrome to the skip list. Keep `head` skipped.
- Decode entities (existing) and collapse runs of blank lines to one.

### 4. Prompting (`providers/prompt.rs`, tool schemas)
- **Always prepend** `Current date: YYYY-MM-DD.` (local date) to the system prompt when any tool is enabled.
- **When web tools are enabled, add a research block (keep it short):**
  ```
  Research procedure:
  1. Run one precise web_search (include the year for "latest" questions).
  2. web_fetch the 1–3 most authoritative URLs from the results — official docs first. You may call several tools in one step.
  3. Answer from fetched content; cite as [n]. If sources conflict, prefer the official one and say so.
  Do not narrate tool use. Only say a tool failed if a tool result contains an error, and quote it.
  ```
- **Tool descriptions:**
  - `web_search`: "Search the web. Returns numbered results with short snippets. Use specific queries; snippets are not enough for exact numbers — fetch the page."
  - `web_fetch`: "Fetch the readable text of a URL (usually from search results). Use for details such as pricing tables, specs and changelogs. Supports `start` for long pages."
- **Profile temperature:** default `temperature: 0.3` for profiles with tools, unless the profile sets its own.

### 5. Separate narration from the answer
- **Backend** (`providers/agent.rs`, `providers/mod.rs`): add `round: u32` to `StreamEvent::Delta`. Text from a round that ends in tool calls is narration; text from the final round is the answer.
- **Frontend** (`src/lib/state/stream.svelte.ts`, `src/components/Message.svelte`):
  - Buffer text per round. When a round ends with tool calls, move its text into that step as a dim one-line note (in `text-tertiary`, after the step line). The answer body only contains final-round text.
  - When streaming in the final round starts with no tool calls, render it as the answer immediately, so there's no added latency.
- **Persistence:** store only the final answer as message `content`; narration isn't sent back in later turns.

### 6. Limits
- `MAX_ROUNDS` 4 → 6. Keep `MAX_SEARCHES = 5` and `MAX_FETCHES` as is.
- **Parallel execution:** tool calls within one round run concurrently, with at most 3 in flight (`futures::join_all` is already available via tokio, so no new crate). A "search + 3 fetches" round then takes one round trip.

### 7. Model guidance (docs, not code)
In `README.md` / settings hints, note that small "flash" models are weak at multi-step tool use. Suggest a `research` profile with a stronger tool-capable model while keeping the cheap model as the default.

## Files
- **New:**
  - `src-tauri/src/providers/trace.rs`
  - `src-tauri/src/tools/fetch_client.rs`: guarded client, redirect policy, Jina fallback. Split it out so `fetch_url.rs` stays under 200 lines.
- **Changed:**
  - `src-tauri/src/tools/fetch_url.rs`, `tools/html.rs`, `tools/search/mod.rs` (description)
  - `src-tauri/src/providers/prompt.rs`, `providers/agent.rs`, `providers/mod.rs` (`Delta.round`)
  - `src-tauri/src/config/mod.rs` + `defaults.rs`: `debug.agentLog`, `search.readerFallback`, tool-profile temperature
  - `src/lib/state/stream.svelte.ts`, `src/components/Message.svelte`, `src/lib/types.ts`, `src/lib/state/palette.svelte.ts` ("Open agent log")

## Verification
- **Unit tests:**
  - `html.rs`: `<main>` preference, a table rendered as pipe rows, title extraction.
  - `fetch_url`: truncation at the byte cap; the `start`/`maxChars` window header; the redirect guard rejects a redirect to `127.0.0.1` (use a local test server, or test the policy closure directly).
  - `prompt.rs`: date line present and research block only when web tools are enabled.
  - `trace.rs`: writes valid JSONL and rotates.
- **Manual:** set `debug.agentLog: true` and use the same prompt with `deepseek-v4.1-flash`. Expect:
  - The trace shows search → parallel fetches of `platform.claude.com` / `anthropic.com` returning 200 (or `via reader`), with non-empty text containing price rows.
  - The answer body has no "Let me…" narration (it appears as dim step notes) and includes a correct pricing table with citations.
  - No "fetch tool isn't available" claim.
- **Regression:** forced search (`Ctrl+W`), the pre-search fallback for non-tool models, and the leak guard from TOOLCALL_LEAK_PLAN.md still behave as before.
