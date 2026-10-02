# SlopChat docs

Documentation for how SlopChat works internally.

| Doc | Covers |
| --- | --- |
| [[agent]] | The research turn: flow, system prompt, injected instructions, tool loop, final answer guarantee. |
| [[limits]] | Every budget, cap and timeout — defaults, clamps, and where each is enforced. |
| [[tools]] | Tool schemas, web-search providers, and the fetch client (SSRF guard, reader fallback). |

## Conventions

- **Cross-doc links** use Obsidian wikilinks (`[[limits]]`).
- **Diagrams** show a plain ASCII version inline and a mermaid version in a collapsed `<details>` block.
- **Code references** use `path::symbol` and link the file, e.g. `providers/prompt.rs::build_messages`. Line numbers are avoided so they don't drift.
- One purpose per file; link instead of duplicating.
- `settings.json` field shapes live in the app's Settings UI and in `src-tauri/src/config/settings.rs`; this folder explains behaviour, not the full config schema.
