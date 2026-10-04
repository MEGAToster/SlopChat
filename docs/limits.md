# Limits and budgets

Every cap and timeout that shapes a research turn. See [[agent]] for how they're used and
[[tools]] for tool-side defaults.

## Turn budgets

Configured under Advanced (`src-tauri/src/config/defaults.rs`, type
`src-tauri/src/config/settings.rs`). Effective clamps are applied where the value is used.

| Setting | Default | Clamp | Enforced by | Effect |
| --- | --- | --- | --- | --- |
| `maxSearches` | 5 | 1–20 | `Budget::from_settings` | `web_search` calls per turn |
| `maxFetches` | 50 | 0–100 | `Budget::from_settings` | `web_fetch` calls per turn |
| `maxFetchedChars` | 60000 | — | `Budget::allow` | Total fetched characters per turn |
| `maxRounds` | 5 | 1–10 | `Budget::from_settings` | Model↔tool rounds before the final pass |
| `responseTimeoutSecs` | 60 | 5–600 | `run_turn` | No-output inactivity per step before it fails |
| `maxTurnSecs` | 300 | 30–3600 | `run_turn` | Whole-turn deadline (see reserve) |
| `reasoningMaxTokens` | 0 | — | `Turn` | OpenRouter `reasoning.max_tokens` (0 = off) |
| `frequencyPenalty` | 0.2 | 0–2 | `run_turn` | OpenRouter sampling penalty, non-reasoning only |
| `presencePenalty` | 0.2 | 0–2 | `run_turn` | OpenRouter sampling penalty, non-reasoning only |

`maxFetches: 0` means fetches are disabled; `maxFetchedChars` counts characters of returned
content. The char cap is only checked when a fetch is requested, so the last allowed fetch
can push the total over.

For the Antigravity CLI (`agy`), `maxTurnSecs` also sets `--print-timeout`, and the process is
hard-killed 10 s later. See [[antigravity]].

## Model limits

| Setting (per model) | Default | Effect |
| --- | --- | --- |
| `temperature` | `None` | Tool turns default to `0.3` when unset; ignored for reasoning models |
| `maxCompletionTokens` | `Some(8192)` | Sent as `max_completion_tokens` to OpenRouter |
| `toolCalling` | `true` | Master switch for tools |
| `webSearch` | `Auto` | `Off` / `Auto` / `Always` |
| `tools` | `[]` | Extra tool names for the model |

## Time model

```
deadline   = now + maxTurnSecs
reserve    = clamp(maxTurnSecs / 4, 15, 45)      // seconds kept for the final answer
start loop only while now + reserve < deadline
final pass runs with deadline = now + reserve
inactivity = responseTimeoutSecs                  // per step, no output
```

The reserve guarantees the final-pass attempts run even when research uses the whole turn,
so a turn always ends with an answer (or an explicit “no answer” note). See [[agent]].

## Budget footer

`Budget::footer()` is appended to **every** tool result — success, error, or refusal:

```
[budget: searches left A/B · fetches left C/D · steps left E/F]
```

- The `fetches` part is omitted when `maxFetches` is 0 or ≥ 50 (i.e. effectively unlimited).
- `steps` is remaining rounds (`maxRounds - rounds`).

## Refusals

A call past a limit is not run; the model gets a `tool` message
`error: <STOP_HINT><budget footer>` and the round is flagged `hit_limit`, which triggers the
one-time wrap-up message. This is our own budget, not a provider failure. `STOP_HINT` tells
the model it may still fetch or answer. Texts: `providers/agent/instructions.rs`.

## Fetch and search defaults

Not in Advanced, but they bound output size: fetch window (`DEFAULT_CHARS` 12 000,
`MAX_CHARS` 20 000, `MAX_BYTES` 2 MB, `MIN_RESULT_CHARS` 50, reader `MIN_TEXT` 500) and
search (`numResults` clamp 1–10, default `search.numResults` 5, `search.snippetChars` 1500).
Details in [[tools]].
