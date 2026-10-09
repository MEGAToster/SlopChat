# Implementation plan: Claude Code (`claude`) as a chat provider

Self-contained plan for an implementing agent. Read `AGENTS.md`, then `docs/antigravity.md` and
`src-tauri/src/providers/antigravity/`. This provider is modelled on the Antigravity one: spawn the
user's own signed-in CLI, stream its NDJSON output and resume its session per chat.

**Phase 0 is mandatory.** It runs live probes against the real binary before any code is written,
because the Antigravity integration found that the docs disagreed with the binary. Facts marked
**(unverified)** come from docs or memory and must be confirmed in Phase 0. If Phase 0 hits a
**stop condition**, stop and report. Don't work around it.

```
cd src-tauri && cargo test && cargo clippy -- -D warnings
npm test && npm run check
```

## Policy (non-negotiable)

Source: <https://code.claude.com/docs/en/legal-and-compliance>, "Authentication and credential
use". The relevant lines:

> Anthropic does not permit third-party developers to offer Claude.ai login into their own
> applications, or to route requests through Free, Pro, or Max plan credentials on behalf of their
> users. Moreover, developers may not collect, store, or intermediate Claude.ai credentials or
> session tokens — sign-in to a Claude account must complete through Anthropic's own flow.
>
> […] Nor does it prevent an end user from signing in to the unmodified Claude Code binary with
> their own Claude subscription […]
>
> Advertised usage limits for Pro and Max plans assume ordinary, individual usage of Claude Code
> and the Agent SDK.

Hard rules that follow from this:

1. **Never read Claude Code's credentials.** No reading `~/.claude/.credentials.json`, the
   `Claude Code-credentials` keychain item, or any token file. No calling the Anthropic API with
   such a token.
2. **No sign-in inside SlopChat.** The user signs in by running `claude` in a terminal, the same
   way `agy` works. SlopChat only reports "not signed in".
3. **Run the binary unmodified,** as installed by the user. No patching and no bundling.
4. **Only the local user's own CLI, for that user.** Nothing that proxies or shares the session.
5. **Disclose it.** `docs/claude-code.md` and the README link the legal page and say that usage
   must stay within ordinary individual use and that the user is responsible for it.
6. **Keep it removable.** Everything lives in `providers/claude_code/` plus thin glue, so the
   provider can be deleted cleanly if the policy changes.

## Decisions already made

- Provider name `claude-code`. Model refs look like `claude-code/<model>`, for example
  `claude-code/sonnet` or `claude-code/opus`.
- Text only in v1. Image attachments are rejected with a clear error, as Antigravity does.
- Cost is always `None`. The subscription isn't billed per token, and `total_cost_usd` is only
  a client-side estimate.
- The profile `fallback` handles "not installed / not signed in / usage limit reached", so a
  profile can fall back to `openrouter/anthropic/...`.
- Deep research and SlopChat's own search/fetch tools are never used with this provider.
- The session-resume logic is shared with Antigravity, not copied (see Step 1).

---

## Phase 0 — Live probes (no code changes)

Run in an empty directory: `mkdir -p "$TMPDIR/cc-probe" && cd "$TMPDIR/cc-probe"`.

- Save every NDJSON output as a fixture under `src-tauri/src/providers/claude_code/fixtures/`.
- **Scrub** home paths, emails, account/org ids and session ids before committing (replace with
  `/home/user`, `user@example.com`, `00000000-…`).
- Record the results in `docs/claude-code.md` with a "confirmed against `claude` x.y.z" note,
  the same way `docs/antigravity.md` does.

The base command, referred to below as `BASE`:
```sh
claude -p --output-format stream-json --verbose --include-partial-messages
```

| # | Probe | Command / method | Record |
| --- | --- | --- | --- |
| P1 | Version | `claude --version` | Exact output format, for version parsing |
| P2 | Plain text | `echo "Say hi in five words" \| BASE > text.ndjson` | All event types and shapes. Confirm the prompt can come from stdin. |
| P3 | Auth source | Inspect `system/init` in `text.ndjson`, then rerun with `ANTHROPIC_API_KEY=bogus` | Whether init reports the auth source (`apiKeySource`?) **(unverified)**. Confirm a set API key overrides the subscription. |
| P4 | `--bare` | `echo hi \| claude --bare -p` | Expected to fail auth with a subscription **(unverified)**. If so, `--bare` is unusable. |
| P5 | Isolation | `BASE --setting-sources "" --strict-mcp-config --mcp-config '{"mcpServers":{}}'` (try variants if a flag is rejected) | The init event's `tools`, `mcp_servers`, `slash_commands` and `plugins` lists. Ask "Quote any CLAUDE.md or memory instructions you were given" to check whether `~/.claude/CLAUDE.md` loads. Check whether user hooks fire. |
| P6 | System prompt | `BASE --system-prompt "You are a pirate. One sentence only."` and ask "What are you and what tools do you have?" | Whether the default Claude Code prompt is **replaced**, not just appended to. Check whether the flag exists in `-p` mode **(unverified)**. |
| P7 | No tools | `BASE --tools ""` | `init.tools` is empty, and a request that needs a tool doesn't hang |
| P8 | Web tools | `BASE --tools WebSearch,WebFetch --allowedTools WebSearch,WebFetch` and ask about today's news → `tool.ndjson` | Tool call and result event shapes. Must not wait on a permission prompt. Check how `--max-turns` interacts. |
| P9 | Resume | Take `session_id` from P2's `result`, then `echo "What did I just ask?" \| BASE --resume <id>` → `resume.ndjson` | Whether context carries over. Whether resume returns the **same or a new** `session_id`, since that decides what gets stored. |
| P10 | Unknown session | `BASE --resume 00000000-0000-0000-0000-000000000000` → `unknown_session.ndjson` + stderr | Error shape and exit code |
| P11 | Models | `--model sonnet`, `opus`, `haiku`, a full model id, and `--model bogus` → `unknown_model.ndjson` | Which names work. Whether a model-list command exists (`claude --help`). |
| P12 | Reasoning | `claude --help \| grep -i -E "effort\|think"`; run an Opus prompt that triggers thinking | Whether there is an effort flag or env var. Whether `thinking_delta` events stream. |
| P13 | Cancel | Start a long answer, send `SIGINT` after 2 s → `cancel.ndjson` + stderr + exit code | What a stop looks like |
| P14 | Not signed in | `CLAUDE_CONFIG_DIR=$(mktemp -d) claude -p hi` **(unverified that this simulates signed-out)** | Error text and exit code, for mapping to `NotSignedIn` |
| P15 | Usage limit | Can't be forced. Search `claude --help` and the docs for the message or event shape, or capture it if it happens naturally. | How to detect it and map it to `Unavailable`, so the fallback kicks in |
| P16 | Usage fields | From `result` | `usage.input_tokens`, `output_tokens`, the cache fields, `num_turns`, `duration_ms` |

### Stop conditions (go/no-go)

Report and stop if any of these is true:

- **P5:** user-level CLAUDE.md, hooks or MCP servers can't be kept out without `--bare`.
- **P6:** the coding-agent system prompt can't be replaced.
- **P2/P7:** a plain chat turn can't run without tool or permission prompts.

### Expected event shapes (to confirm)

Confirm these in P2/P8/P9 **(unverified)**:

- `{"type":"system","subtype":"init","session_id","model","tools":[…],"mcp_servers":[…],…}`
- `{"type":"stream_event","event":{API streaming event}}`. Text arrives as `content_block_delta`
  with `delta.type: "text_delta"`, and thinking as `"thinking_delta"`.
- `{"type":"assistant","message":{"content":[text | tool_use]}}`
- `{"type":"user","message":{"content":[tool_result]}}`
- `{"type":"result","subtype":"success"|"error_…","is_error","result","session_id","usage":{…},"total_cost_usd",…}`

---

## Step 1 — Share the CLI session logic

Antigravity's `session.rs` handles which turns are already inside the CLI's conversation, and it
isn't specific to `agy`.

1. Move `src-tauri/src/providers/antigravity/session.rs` to `src-tauri/src/providers/cli_session.rs`.
   Update the imports in `antigravity/mod.rs`.
2. Add `provider: String` to `Stored`, with `#[serde(default = "default_provider")]` where
   `default_provider()` returns `"antigravity"`, so existing rows decode.
   - `plan(request, stored, provider)` returns `Fresh` when `stored.provider != provider`.
   - Add a test for the provider mismatch.
3. Keep using the `chats.agy_session` column; it now stores any CLI provider's pointer. Note this
   in a short *why* comment at `store/chats.rs` `get_agy_session`. Don't rename the column.

## Step 2 — `providers/claude_code/` module

Mirror the Antigravity layout. Each file stays under 200 lines.

| File | Contents |
| --- | --- |
| `mod.rs` | `PROVIDER = "claude-code"`, `run()` glue. Copy the shape of `antigravity::run`: text-only guard → `cli_session::plan` → exec → re-seed once on unknown session → store session → `Usage` + `Done`. |
| `binary.rs` | Locate and check the version. Probe `~/.local/bin/claude`, `~/.claude/local/claude`, `/opt/homebrew/bin/claude` and `/usr/local/bin/claude` first (a Finder-launched app has no shell `PATH`), then walk `PATH`. Set `MIN_VERSION` to the version used in Phase 0. `status()` matches `antigravity::binary::Status`. |
| `exec.rs` | `ClaudeError` (`NotInstalled`, `TooOld`, `NotSignedIn`, `UsageLimit`, `UnknownSession`, `UnknownModel`, `Cancelled`, `TimedOut`, `Failed`) with `message()`, plus `ExecOptions { model, session, prompt, system, web, timeout }`. Keep it free of the store and Channel, like Antigravity. |
| `process.rs` | Spawn and stream. See the invocation below. |
| `events.rs` | `parse_line` → a `ClaudeEvent` enum covering only the fields used. Unknown types are ignored, never an error. |
| `mapper.rs` | `ClaudeEvent` → `StreamEvent`: text deltas → `Delta`; thinking → `Reasoning`; tool_use/tool_result → `ToolCall`/`ToolResult` (summary = short text); `result` → returned to `run`. |
| `models.rs` | `list()`. If P11 found a list command, use it. Otherwise return a static `sonnet`, `opus`, `haiku` list when the binary is present. |

### Invocation

Adjust to Phase 0 findings:

```
claude -p --output-format stream-json --verbose --include-partial-messages
       --model <model> --system-prompt <system> [--resume <id>]
       --tools "" | --tools WebSearch,WebFetch --allowedTools WebSearch,WebFetch
       --max-turns <1 | N> <isolation flags from P5>
stdin: the prompt as plain text, then close stdin
```

- **Working directory:** an empty `crate::paths::data_dir().join("claude-code").join("workspace")`.
  Copy `antigravity::process::workspace`.
- **Environment:** `env_remove("ANTHROPIC_API_KEY")` and `env_remove("ANTHROPIC_AUTH_TOKEN")`, so
  the user's own subscription sign-in is used (P3). Add a *why* comment.
- **System prompt:** `--system-prompt` gets `prompt::system_text(request)`. When resuming, still
  pass it if P9 shows it's needed.
- **Web:** `request.web_search` is `Off` → `--tools ""`. `Auto` or `Deep` → the WebSearch/WebFetch
  allowlist, but only if P8 passed. Otherwise always `--tools ""`, and the web toggle is disabled
  for this provider.
- **Timeout:** use `advanced.maxTurnSecs`, enforced by SlopChat, which kills the process (there's
  no CLI flag). Reuse the `KILL_GRACE` pattern from `antigravity/process.rs`.
- **Cancel:** on Stop, send `SIGINT`, then kill. Map the outcome to `Cancelled`, which `run` turns
  into `Done`.
- **Errors:** map stderr and `result` errors to `ClaudeError` using the P10/P11/P13/P14/P15
  fixtures.

## Step 3 — Glue (thin)

1. `providers/mod.rs`:
   - Add `pub mod claude_code; pub mod cli_session;`.
   - Add `claude_code::models::list()` to the `tokio::join!` in `list_all`.
   - `reasoning_for` returns `None` for `claude-code`, unless P12 found an effort control. In that
     case return the supported levels and pass them through `ExecOptions`.
2. `providers/agent/turn_run.rs`:
   - Add a `claude_code::PROVIDER` branch next to the Antigravity one.
   - Add `claude_code_error()`: `NotInstalled | TooOld | NotSignedIn | UsageLimit | UnknownModel`
     → `TurnError::Unavailable` (triggers the profile fallback), everything else → `Failed`.
3. `providers/agent/research.rs:18`: also return `None` for `claude-code`.
4. `config/defaults.rs:211-227`: add a `"claude-code"` provider entry exactly like `"antigravity"`.
5. `commands/providers.rs`: add a `claude_code_status` command that delegates to
   `claude_code::binary::status()`. Register it in `main.rs`.

## Step 4 — Frontend

1. `src/lib/status.ts`:
   - Add `isCliProvider(model)` (true for `antigravity` and `claude-code`) and use it wherever
     `isAntigravity` currently means "no SlopChat tools / no reasoning button":
     `FooterControls.svelte`, `Footer.svelte:32`, `ModelEditor.svelte:63`.
   - Keep `webMode` returning `"off"` for `claude-code` unless Step 2 enabled web.
2. `src/lib/usageInfo.ts`: label `claude-code` like `antigravity`, and update `usageInfo.test.ts`.
3. `ModelEditor.svelte:36`: add a placeholder model for the `claude-code` provider (`sonnet`).
4. New file `src/components/ClaudeCodeSection.svelte`, modelled on `AntigravitySection.svelte`:
   - Status line (path · version) and Refresh.
   - Install hint that links the official install docs, not a copied command.
   - "Run `claude` once in a terminal and sign in."
   - One muted line: "Uses your own Claude subscription through the official CLI. Usage limits
     assume ordinary individual use."
   - Add `claudeCodeStatus()` to `ipc.ts` and the type to `types.ts`. Mount the section in
     `Settings.svelte` after Antigravity.

## Step 5 — Tests

- `events.rs`: parse every Phase 0 fixture. Unknown event types are ignored.
- `mapper.rs`: from the fixtures, check the expected `StreamEvent` sequence (text, thinking,
  tool, result).
- `process.rs`: reuse the fake-binary approach from `antigravity/process/fake_binary.rs`.
  - A script that prints a fixture and exits.
  - Assert the arguments passed: `--resume` only when resuming, `--tools ""` when web is off.
  - Assert `ANTHROPIC_API_KEY` is **not** in the child's environment.
  - Cover the timeout kill and cancel.
- `cli_session.rs`: the existing session tests plus the provider-mismatch test.
- `binary.rs`: version parsing against the P1 output.

## Step 6 — Docs

- New `docs/claude-code.md`, structured like `docs/antigravity.md`: requirements, invocation,
  output events (confirmed version), resume table, isolation, quirks, troubleshooting, and the
  **Policy** section with the legal-page link and the hard rules above. Link it from `docs/README.md`.
- README: add a bullet under **Providers** with a one-line policy note and link.
- Final check: all commands under **Commands** in `AGENTS.md`, and no new dependencies.
