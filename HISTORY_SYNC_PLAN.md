# SlopChat — chat history sync (plan)

Continue the same chat on another device by merging a portable, id-keyed history
bundle from a configurable sync folder. Any file transport works (Syncthing, git,
rsync, cloud, USB); the app does not depend on a particular one.

## Decisions
- Local SQLite remains the single source of truth; the folder is a mirror.
- Merge runs on **startup**, optionally every **N minutes** (opt-in), and **export on exit**.
- Sync is **disabled by default**; interval default `0` (startup + exit only).
- Sync folder path is **configurable**, default empty (user must pick one to enable).
- **Deletes and pins propagate**; attachments are included.
- Deleted items are kept for **60 days** (tombstone retention), then purged.
- `context.md` is included in the bundle; `themes/` is not.
- Cross-platform (macOS ↔ Linux).

## Phase 0 — prerequisites

### 1. Globally unique, sortable ids
`store/mod.rs::new_id` currently uses `time + per-run counter` and can collide across
devices. Switch to UUIDv7/ULID (`format!("{prefix}{uuid}")`, prefixes `c`/`m`/`a`
kept). Add the chosen crate to `Cargo.toml`.

### 2. Relative attachment paths
- `attachments.path` stores only the filename (`<id><ext>`); resolve against
  `attachments_dir()` in `attachments::read` and when serialising to the frontend.
- Migration V5 (Rust, row-by-row) rewrites existing absolute paths to their basename.
- Frontend already uses `basename(file.path)` and passes it back to `read_attachment`.

### 3. Tombstones + timelines
- `chats`: add `deleted_at INTEGER` (nullable).
- `messages`: add `updated_at INTEGER NOT NULL` (backfill `= created_at`) and
  `deleted_at INTEGER`.
- `chats::delete` → soft delete (set `deleted_at`, bump `updated_at`).
- `messages::rewind` → set `deleted_at` on the removed tail and bump `updated_at` on
  the edited message, instead of hard `DELETE`.
- FTS triggers keep indexing; `store/search.rs` and all read queries filter
  `deleted_at IS NULL`.
- Migration V6 (SQL) adds the columns; `user_version` → 6.

### 4. Pin exclusivity
`chats.pinned` is already exclusive within a device. After any merge, keep only the
pinned chat with the newest `updated_at`; clear the rest.

## Phase 1 — bundle format
```
<sync folder>/
  manifest.json                 # { schema: 1, deviceId, exportedAt }
  context.md                    # optional mirror of the user context file
  chats/<chatId>.json           # { schema, chat, messages: [ … ] }
  attachments/<fileId><ext>
```
- Ids lowercased for case-insensitive filesystems; extensions sanitised.
- Chat files are full snapshots, including tombstones (`deletedAt`).
- Atomic writes (temp + rename), `manifest.json` last.
- `context.md` is a single file: newest `mtime` wins on merge/export.

## Phase 2 — merge (union, newest wins)
For each `chats/<id>.json`, on a background thread holding the store lock:
1. **Chat**: insert if absent; else apply the side with the larger `updated_at`
   (`title`, `profile`, `pinned`, `deleted_at`).
2. **Messages**: upsert by id; insert if absent, else take the larger `updated_at`
   (covers edits and rewind tombstones).
3. **Attachments**: copy missing files from the bundle into local `attachments/`;
   upsert rows. Never delete local files.
4. Never delete local rows absent from a bundle — deletions only via tombstones.
5. `context.md`: adopt the bundle copy when it is newer.
6. Enforce pin exclusivity; rebuild FTS once at the end.
7. Emit `history://changed`; the frontend refreshes the chat list / active chat.
- Skip files not matching `<uuid>.json` (ignores Syncthing `.sync-conflict*`); skip
  malformed files with a logged warning.

## Phase 3 — export
- Snapshot all chats (including tombstones) to `chats/<id>.json`; copy referenced
  attachment files not already present; mirror `context.md`.
- Skip writes whose serialised content is unchanged (compare bytes) to avoid churn.
- Run on startup after merge, on the interval, on exit, and manually.

## Phase 4 — retention (60 days)
- A maintenance pass (startup and on the interval) removes chats/messages with
  `deleted_at` older than 60 days, plus their attachment files, and drops them from
  the export on the next sync.
- Known limitation (documented): a device offline longer than 60 days can resurrect
  a delete; acceptable for v1.

## Phase 5 — triggers, settings, UI
- **Startup**: after `Store::open` + `config::init`, if enabled: merge → export.
- **Interval**: task every `intervalMinutes` (0 = disabled) doing merge → export.
- **Exit**: best-effort export before process exit (`RunEvent::ExitRequested`).
- **Manual**: "Sync now" palette action + Settings button via `sync_now` command,
  guarded so concurrent runs no-op.
- Settings additions:
  ```jsonc
  "sync": { "enabled": false, "folder": "", "intervalMinutes": 0 }
  ```
- Frontend: `Settings.sync` type, Settings section (enable, folder picker, interval),
  commands `get_sync` / `set_sync` / `sync_now`, and a `history://changed` listener
  that refreshes state.

## Cross-platform
- Id-based lowercased filenames; `Path::join` only.
- Attachment paths relative; absolute reconstructed per machine.
- Expand a leading `~` in the folder path at use time; refuse a folder inside the
  data/config dirs to avoid recursion.

## Out of scope (v1)
- Continuous/live sync, conflict-free per-row merge of simultaneous edits.
- Themes sync.
- Server or Syncthing-specific integration.

## Files
- Rust: `store/mod.rs`, `store/migrations.rs` (V5/V6), `store/chats.rs`,
  `store/messages.rs`, `store/attachments.rs`, `store/search.rs`; new `store/sync/`
  module (`plan.rs`, `export.rs`, `merge.rs`, `maintenance.rs`) kept small;
  `config/settings.rs` + `config/defaults.rs` (sync settings); `config/write.rs`
  (persist `sync`); `commands.rs`; `main.rs` (hooks).
- Frontend: `lib/types.ts`, `lib/ipc.ts`, Settings UI, palette action, chat refresh.

## Testing
- Rust: V5 path migration, V6 columns, UUID format, soft-delete + FTS filtering,
  merge union / newest-wins / tombstones / pin exclusivity / attachment copy,
  60-day purge; a two-DB simulation (A export → B merge → mutate B → A merge).
- Manual: two temp folders as two devices, including a deleted chat and a rewound chat.

## Effort
- Phase 0: ~1.5–2.5 days
- Phases 1–3: ~1.5–2 days
- Phases 4–5 + UI + tests + docs: ~1.5–2 days
- **Total ≈ 5–7 dev-days**

## Dependency
Chats store `chat.profile` by name; sync the model split (`models.json`) alongside so
a continued chat resolves the same profile. Falls back gracefully if missing.
