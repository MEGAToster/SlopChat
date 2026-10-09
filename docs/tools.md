# Tools

The two tools the agent can call, and how they execute. Budgets are in [[limits]]; the loop
that calls them is in [[agent]].

## Schemas

Definitions: [`tools/search/mod.rs`](../src-tauri/src/tools/search/mod.rs)::`schema` and
[`tools/fetch_url.rs`](../src-tauri/src/tools/fetch_url.rs)::`schema`. Schemas must stay
identical for the whole turn — tool use is stopped through results and `tool_choice`, never
by dropping definitions mid-turn.

`web_search`:

```json
{
  "name": "web_search",
  "description": "Search the web. Returns numbered results with short snippets. Use specific queries; snippets are not enough for exact numbers — fetch the page.",
  "parameters": {
    "type": "object",
    "properties": {
      "query":      { "type": "string" },
      "numResults": { "type": "integer", "minimum": 1, "maximum": 10 }
    },
    "required": ["query"]
  }
}
```

`web_fetch`:

```json
{
  "name": "web_fetch",
  "description": "Fetch the readable text of a URL (usually from search results). Use for details such as pricing tables, specs and changelogs. Supports `start` for long pages.",
  "parameters": {
    "type": "object",
    "properties": {
      "url":      { "type": "string" },
      "start":    { "type": "integer", "minimum": 0, "description": "Character offset to continue from on long pages" },
      "maxChars": { "type": "integer", "minimum": 1, "maximum": 20000 }
    },
    "required": ["url"]
  }
}
```

## web_search

Calls a paid search API — the only tool that does. [`tools/search/mod.rs`](../src-tauri/src/tools/search/mod.rs).

It walks `search.providers` **in order** and returns the first success:

1. Skip providers without a stored key, or whose free allowance is used
   (`budget::is_exhausted` / `budget::allowed`).
2. On success: `budget::record(units, cost)` and format the numbered results.
3. On quota/auth failure (`SearchError::Exhausted`): mark exhausted, try the next.
4. On other failure: record the reason, try the next.

If all providers fail, the app notifies “search unavailable” and returns an error listing
each reason. Results are numbered across the turn (source counter carries into citations).

### Providers

Ordered `searxng → exa → tavily → brave → firecrawl → serper`. SearXNG is skipped until a
base URL and an `X-API-Key` token are set. Default free allowances (`config/defaults.rs`):

| Provider | Default limit | Kinds |
| --- | --- | --- |
| searxng | — | self-hosted, `X-API-Key` |
| exa | `monthlyUsd: 10` | dollar budget |
| tavily | `monthly: 1000` | request count |
| brave | `monthlyUsd: 5` | dollar budget |
| firecrawl | `total: 500` | flat count |
| serper | `total: 2500` | flat count |

Providers are **fallbacks only**. Allowance tracking is best-effort; usage past a free
allowance (pay-on-demand) will be charged by the provider — see the README web-search
billing section.

### SearXNG

Set the instance URL and its `X-API-Key` token in Settings → Web search (the URL is stored as
`search.searxng.baseUrl`; the token lives in the OS keychain, or in `SEARXNG_API_KEY`). Every
request carries `X-API-Key: <token>`. Public instances usually disable the JSON API, so run
your own:

```sh
docker run -d -p 8080:8080 searxng/searxng
```

Then add `json` to `search.formats` in the instance's `settings.yml`. A non-JSON response
surfaces as `SearXNG JSON output is disabled — add json to search.formats in settings.yml`,
and a `401` surfaces as `SearXNG rejected the API key (X-API-Key)`.

The instance's own limiter is often off, so the client spaces requests out itself: at most one
request per `search.searxng.minIntervalMs` (default `700`, `0` disables), plus a uniform
0–30 % jitter so the traffic does not look machine-like. This keeps bursts from getting the
instance banned by its upstream engines. SearXNG is free, so keep it first to research without
paid search.

On launch the app runs one test query through the same throttled path when a URL and token are
configured, and shows a warning toast if it fails. The check is skipped when there is no URL or
no readable token (for example a locked keychain), so it never prompts or blocks startup.

## web_fetch

**Structurally provider-free**: [`tools/mod.rs`](../src-tauri/src/tools/mod.rs)::`run` calls
`fetch_url::run(args, reader_fallback)`, so a fetch never spends search credits.
[`tools/fetch_client.rs`](../src-tauri/src/tools/fetch_client.rs) does the work:

- Own `reqwest` client with a desktop Safari user agent. The content type only selects HTML
  extraction vs. raw text; binary bodies (PDFs, images) are not rejected and are decoded as
  UTF-8.
- **SSRF guard** ([`tools/ssrf.rs`](../src-tauri/src/tools/ssrf.rs)): parse the URL with the
  same WHATWG rules reqwest uses, block loopback/private/link-local/`0.0.0.0/8`/IPv4-mapped
  addresses, resolve the host asynchronously, pin the resolved IP, and follow redirects by
  hand (max 5) re-resolving and re-pinning every hop.
- Downloads at most **2 MB**, then extracts readable text (prefers `main`/`article`, keeps
  tables and the title) via [`tools/html.rs`](../src-tauri/src/tools/html.rs).
- Returns a `start`/`maxChars` window (`DEFAULT_CHARS` 12 000, `MAX_CHARS` 20 000) with a
  header showing the URL and `chars X-Y of Z`.
- **Reader fallback** (`search.readerFallback`, default true): if the page is blocked or has
  under 500 chars of text, retries through the free `r.jina.ai` reader. When it is on and the
  result still has under 50 chars (`MIN_RESULT_CHARS`), the fetch returns an error instead of
  a useless near-empty page.

`web_search` snippets are reminders, not sources; the system prompt pushes the model to
fetch pages for exact numbers (see [[agent]]).
