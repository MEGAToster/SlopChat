# SlopChat — project rules

## Ground rules (non-negotiable)
1. **Lightweight.** Every dependency must earn its place. Prefer the standard library and small crates/packages. No UI component libraries, no CSS frameworks, no state-management libraries.
   - Targets: release bundle under ~15 MB, idle RAM under ~100 MB in total.
2. **Small files.** One purpose per file. Aim for under 200 lines and split before 300.
3. **Comments:**
   - Minimal and concise, and they explain *why*, not *what*.
   - Never write history ("changed from X", "previously", "now uses", "fixed"), TODO chatter, or commented-out code.
4. **Thin layers.**
   - Tauri commands only validate and delegate.
   - Platform-specific code lives only in `panel/` and `platform/` modules.
5. **No secrets on disk.** API keys live in the OS keychain only.
6. **Keep the codebase small.** Don't build abstractions for things that exist once. No speculative features beyond this document.

## Commands
- Install: `npm install`
- Dev: `npm run tauri dev`
- Build: `npm run tauri build`
- Test: `cd src-tauri && cargo test`, `npm test`
- Lint: `cd src-tauri && cargo clippy -- -D warnings`
- Typecheck frontend: `npm run check`
