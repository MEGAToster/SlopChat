# SlopChat — Multiple web search providers (implementation plan)

Follow CLAUDE.md: small files (<200 lines, split before 300), minimal "why" comments, no history comments, no new dependencies, secrets only in the keychain, thin Tauri commands.

## Context
- The `web_search` tool from WEBSEARCH_PLAN.md exists (uncommitted), but it's hardwired to Exa (`src-tauri/src/tools/web_search.rs`).
- Goal: support **Exa → Serper → Tavily → Brave**, in that fixed default order. Use each provider only while its **free allowance** lasts, and fall back to the next one when it's used up or the API reports a quota or auth error. Once every free allowance is exhausted, search is off until allowances refill. The app never spends beyond the free tiers.
- The tool-calling loop, `fetch_url`, citations, sources UI and budget table stay as they are. Only the search backend layer and its settings, usage tracking and UI change. No SearXNG.

## Providers
| Order | Provider | Free allowance (default local limit) | Key (keychain) | Endpoint |
|---|---|---|---|---|
| 1 | **Exa** | $10 credit/month, resets on the 1st → `monthlyUsd: 10` | `exa` | existing |
| 2 | **Serper** (Google results) | 2,500 queries **once** on signup; credits expire after ~6 months → `total: 2500` | `serper` | `POST https://google.serper.dev/search` |
| 3 | **Tavily** | 1,000 credits/month, refills → `monthly: 1000` | `tavily` | `POST https://api.tavily.com/search` |
| 4 | **Brave** | $5 credit/month (~1,000 queries at $5/1k) → `monthlyUsd: 5`; attribution required | `brave` | `GET https://api.search.brave.com/res/v1/web/search` |

Providers without a key are skipped silently.

### Request and response mapping (each backend returns `Vec<Hit>` plus usage)
- **Exa:** move the existing request/response code unchanged into its own backend file.
  - Usage: `costDollars.total` → `cost_usd`.
- **Serper:**
  - Request: header `X-API-KEY: <key>`, body `{ "q", "num": n }`.
  - Response: `organic[]` with `title`, `link`, `snippet` and `date?`.
  - Prepend `answerBox.answer` / `answerBox.snippet` and `knowledgeGraph.description` as hit 0 when present.
  - Usage: `credits` field in the response (default 1) → `units`, with cost 0.
- **Tavily:**
  - Request: header `Authorization: Bearer <key>`, body `{ "query", "max_results": n, "search_depth": "basic", "topic": "general" }`.
  - Response: `results[]` with `title`, `url`, `content` and `published_date?`.
  - Usage: 1 unit per basic search, cost 0.
- **Brave:**
  - Request: headers `X-Subscription-Token: <key>`, `Accept: application/json`; query `q`, `count=n`.
  - Response: `web.results[]` with `title`, `url`, `description` (strip `<strong>` tags with `tools/html.rs`) and `age?`.
  - Usage: 1 unit, `cost_usd = 0.005`.

Snippets from Serper and Brave are short. The tool description tells the model to call `fetch_url` (free) on the most relevant results when it needs detail.

## 1. Settings (`src-tauri/src/config/mod.rs`, `config/defaults.rs`)
Replace `SearchSettings.provider: String` with an ordered list plus per-provider free limits:
```json
"search": {
  "providers": ["exa", "serper", "tavily", "brave"],
  "numResults": 5,
  "limits": {
    "exa":    { "monthlyUsd": 10 },
    "serper": { "total": 2500 },
    "tavily": { "monthly": 1000 },
    "brave":  { "monthlyUsd": 5 }
  },
  "exa": { "type": "instant", "contents": "highlights", "maxCharacters": 1500 }
}
```
- **Limit struct:** `Limit { monthly: Option<u32>, monthly_usd: Option<f64>, total: Option<u32> }`. Any combination is allowed, and every field that's set must hold for the provider to be used.
- **Removed:** `monthlyBudget` is replaced by `limits.exa.monthlyUsd`. Map a legacy `monthlyBudget` onto it.
- **Backward compatibility:**
  - A legacy `"provider": "exa"` key is ignored in favor of the default list. Add a unit test for this.
  - Move `type`, `contents` and `maxCharacters` into a nested `ExaSettings`. Accept the old top-level keys via a small post-deserialize fixup.
- **Keys:** resolve them through the existing `providers::api_key(&settings, name)` in `src-tauri/src/providers/mod.rs` (handles `env:VAR` and the keychain). Fall back to the env var `<NAME>_API_KEY`, which replaces the Exa-only `exa_key()` helper.

## 2. Backend layer (`src-tauri/src/tools/search/`)
Replace `tools/web_search.rs` with:
```
tools/search/
  mod.rs       schema(), run(): provider loop, fallback, formatting into numbered content + sources
  hit.rs       struct Hit { title, url, snippet, published }, struct Batch { hits, units, cost_usd }
  exa.rs       serper.rs    tavily.rs    brave.rs
```
- **Backend signature** (plain functions, no trait objects):
  ```rust
  pub async fn search(query: &str, n: u32, ctx: &BackendCtx) -> Result<Batch, SearchError>
  ```
  - `BackendCtx` holds the settings, the resolved key and the shared `crate::providers::client()`.
  - Timeout 15 s.
- **Errors:** `enum SearchError { Exhausted(String), Failed(String) }`.
  - `Exhausted` means the free allowance is used up or unusable: HTTP 401/402/403/429/432/433, or a body mentioning credits, quota or limit. Examples: Serper `"Not enough credits"`, Tavily 432 "plan limit", Exa 402.
  - When a provider reports `Exhausted`, **mark it exhausted locally** for the rest of the month (Serper: until its key changes). This avoids re-hitting it on every search: insert into `search_exhausted(provider, month)`, which is cleared when a new key is stored for that provider.
  - `Failed` means a network error or bad JSON. Fall through to the next provider this time only, without marking.
- **Loop** (`search/mod.rs::run`):
  1. For each name in `settings.search.providers`: skip it if it has no key, is marked exhausted, or fails `budget::allowed` (§3). Otherwise call its backend.
  2. On success, `budget::record`, then format exactly like today's `web_search.rs`: `[n] title — url (published)` plus the indented snippet, numbered from `ctx.start_index`. Set `summary` to `"5 results · serper"`.
  3. If every provider is skipped or fails, return `Err("web search unavailable: exa: free credit used; serper: no key; tavily: quota reached; brave: no key")`. The agent loop passes it back to the model, which answers without search. Show a toast once per session: "All free search allowances used — search resumes when they refill."
- **Formatting reuse:** strip tags from snippets with `tools/html.rs` (expose `strip_tags` if needed) and truncate each to `exa.maxCharacters`.
- **`tools/mod.rs`:** route `web_search` to `search::schema()` / `search::run()`. Tool description: *"Search the web. Results are short snippets; call fetch_url on the most relevant URLs for details."*

## 3. Usage tracking per provider
- **Migration v3** (`src-tauri/src/store/mod.rs`, `PRAGMA user_version` 3):
  ```sql
  CREATE TABLE search_usage_v3 (month TEXT, provider TEXT, requests INTEGER, units INTEGER, cost REAL,
                                PRIMARY KEY (month, provider));
  INSERT INTO search_usage_v3 SELECT month, 'exa', requests, requests, cost FROM search_usage;
  DROP TABLE search_usage;
  ALTER TABLE search_usage_v3 RENAME TO search_usage;
  CREATE TABLE search_exhausted (provider TEXT PRIMARY KEY, month TEXT);
  ```
- **`tools/budget.rs`:**
  - `record(app, provider, units, cost)`.
  - `monthly(app, provider) -> (requests, units, cost)` for the current month.
  - `total_units(app, provider)` summed over all months, for Serper's one-time allowance.
  - `allowed(app, provider, limit) -> bool` checks `monthly` against units, `monthlyUsd` against cost (keeping a $0.02 headroom), and `total` against lifetime units.
  - `mark_exhausted` / `is_exhausted` / `clear_exhausted`. `store_api_key` in `commands.rs` calls `clear_exhausted` for search providers, and also resets Serper's lifetime count by recording a "reset" marker, so a new Serper account gets a fresh 2,500.
- **`search_usage` command** (`src-tauri/src/commands.rs`, `#[tauri::command(async)]`): returns `[{ provider, requests, units, cost, limit, exhausted, hasKey }]` in configured order.

## 4. Frontend
- **`Settings.svelte`:** replace the hardcoded Exa key row with a "Web search" section.
  - One `ApiKeyRow` per provider, in order: Exa, Serper, Tavily, Brave.
  - Each row shows its placeholder ("stored" / `tvly-…`), a save action, and a hint with the free allowance.
  - Show the order read-only as `exa → serper → tavily → brave`, with providers that have no key or are exhausted dimmed. The order can be changed in `settings.json`.
  - Extract `src/components/ApiKeyRow.svelte` so `Settings.svelte` drops back under 200 lines (it's 237 now).
- **Palette "Search usage"** (`src/lib/state/palette.svelte.ts`): one line per keyed provider, e.g. `exa $3.10/$10 · serper 412/2500 total · tavily 0/1000 · brave —`. Exhausted providers are marked `used up`.
- **Types:** update `searchUsage()` in `src/lib/types.ts` and `src/lib/ipc.ts`.
- **Step line:** the existing summary shows which provider answered (`· serper`).

## 5. Tests
- **Rust, one parser test per backend.** Each takes a JSON fixture as an inline string and asserts the `Hit` titles, urls and snippets. Serper: `answerBox` becomes hit 0. Brave: `<strong>` tags are stripped.
- **Fallback test:** make the backend call injectable (a closure passed to the provider loop), then check:
  - Exa `Exhausted` → Serper is used and Exa is marked exhausted for the month.
  - Serper lifetime `total` reached → skipped without a call.
  - `Failed` → next provider, not marked.
  - All out → combined error.
- **Budget test:** `allowed` respects `monthly`, `monthlyUsd` and `total`. Storing a new key clears the exhausted mark.
- **Config test:** legacy `provider` / `monthlyBudget` / top-level Exa keys map onto the new shape.
- **Migration test:** v2 DB with an Exa usage row → v3 with the row preserved under `exa`.

## 6. Files
- **New:** `src-tauri/src/tools/search/{mod,hit,exa,serper,tavily,brave}.rs`, `src/components/ApiKeyRow.svelte`.
- **Removed:** `src-tauri/src/tools/web_search.rs` (logic moved into `search/mod.rs` + `search/exa.rs`).
- **Changed:**
  - `src-tauri/src/tools/mod.rs`, `tools/budget.rs`, `tools/html.rs` (`strip_tags`)
  - `src-tauri/src/config/mod.rs` + `defaults.rs`
  - `src-tauri/src/store/mod.rs` (v3), `src-tauri/src/commands.rs`
  - `src/components/Settings.svelte`, `src/lib/state/palette.svelte.ts`, `src/lib/types.ts`, `src/lib/ipc.ts`

## 7. Verification
- Run `cd src-tauri && cargo test && cargo clippy -- -D warnings`, `npm run check`, `npm test`.
- **Manual** (`npm run tauri dev`):
  1. With all four keys stored, ask a current-events question: the step line shows `· exa`.
  2. Set `limits.exa.monthlyUsd: 0`: the next search uses `· serper` without calling Exa.
  3. Store a bogus Exa key: Exa returns 401, is marked exhausted, and Serper answers. Storing the real key again clears the mark.
  4. Set `limits.serper.total: 0` as well → `· tavily`. Set `limits.tavily.monthly: 0` as well → `· brave`.
  5. Set all limits to 0: the answer arrives without search, with a one-time toast and the tool error in the step line.
  6. Palette "Search usage" matches each provider's dashboard.
