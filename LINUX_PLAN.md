# SlopChat — Linux support (implementation plan)

Follow CLAUDE.md: small files (<200 lines), minimal "why" comments, no history comments, no new crates, thin commands, platform code only in `panel/` and `platform/`.

Milestone 8 of PLAN.md. macOS is the reference platform; this plan makes Linux build, run and toggle.

## Acceptance (PLAN.md §7.8)
- `panel/linux.rs`: always-on-top, skip-taskbar, undecorated.
- `slopchat toggle` for Wayland compositors.
- README section with WM float rules (Hyprland, sway, i3).
- Builds and toggles on Linux (X11 global hotkey and Wayland via CLI).

## Current state
- `panel/linux.rs` builds the window `decorations(false)`, `always_on_top(true)`, `skip_taskbar(true)`, `resizable(true)`, `visible(false)`; implements `init/show/hide/is_visible/set_window_style/resize`. `panel/mod.rs` provides the platform-agnostic `set_expanded/toggle_expanded/is_expanded/is_suppressed/work_area/compact_position/Sticky`.
- `platform.rs::open_path` uses `xdg-open` off macOS. `tauri.conf.json` has `app.windows: []` (the window is created in code). The `main-panel` capability exists.
- `config/keybinds.rs::apply` reports failures without aborting; `config/mod.rs::apply_toggle` routes them to a toast.

## Gaps
1. No `cli.rs`; the single-instance callback (`main.rs:22`) ignores `argv` and always `panel::show`. PLAN §6 expects `cli.rs` for `slopchat toggle`.
2. No hide-on-blur on Linux. macOS hides from the panel's `window_did_resign_key` (`panel/macos.rs:80`); `linux.rs` has no equivalent, so the panel never hides on focus loss.
3. No `README.md` (build deps, autostart, WM rules).
4. Unverified: `linux.rs` only compiles off macOS, so a macOS `cargo check` never builds it, and there is no CI.

## Platform findings
- **Single instance uses DBus on Linux**, not a lockfile. The first instance publishes `org.{identifier, dots/dashes → underscores}.SingleInstance`; later instances message it and exit. Here `com.slopchat.app` → `org.com_slopchat_app.SingleInstance`. The `init` closure is `(app, args: Vec<String>, cwd: String)` and the plugin must be registered first (it is). Works under Wayland. Snap/Flatpak need `--talk-name`/`--own-name` for that service.
- **`tauri-plugin-global-shortcut` does not work on Wayland** (X11 `XGrabKey`). CLI toggle is therefore the required Wayland path. A failed registration currently shows a config-error toast.
- **Wayland compositors ignore window hints**: `set_position`, keep-above, and `skip_taskbar` are policy. Sizing, show/hide and focus generally work. Positioning/centering must come from WM float rules.
- **`keyring` v4 on Linux** needs a Secret Service (GNOME Keyring / KWallet via DBus) or falls back to keyutils; without one, API-key storage fails.
- **Tray** needs `libayatana-appindicator`; **build** needs `webkit2gtk-4.1`, `libsoup3`, `librsvg`, a C compiler (rusqlite bundled), and `patchelf` + `linuxdeploy` for AppImage.
- The attachment picker is an HTML `<input type="file">` (`Composer.svelte:108`) wrapped in `set_suppress_hide` (`Composer.svelte:48`), so hide-on-blur must honour `is_suppressed`.

## Fix

### 1. CLI toggle
New `src-tauri/src/cli.rs` (~30 lines):
```rust
pub enum Command { Toggle }
pub fn parse(argv: &[String]) -> Option<Command>; // argv.get(1) == "toggle"
pub fn run(app: &AppHandle, argv: &[String]);      // Toggle => panel::toggle(app)
```
`main.rs`:
- `mod cli;`
- `.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| cli::run(app, &argv)))`.
- Cold start: in `.setup`, read `std::env::args().collect::<Vec<_>>()`; if `Toggle`, defer `cli::run(app.handle(), &argv)` via `app.run_on_main_thread` (the window is created by `panel::init` moments earlier). No-arg launch keeps today's hidden-until-hotkey behaviour.

### 2. `panel/linux.rs`
- Attach a focus handler in `init` so the panel hides on focus loss unless sticky or suppressed:
```rust
use tauri::WindowEvent;
let handle = app.clone();
window.on_window_event(move |event| {
    if let WindowEvent::Focused(false) = event {
        if !super::is_sticky(&handle) && !super::is_suppressed(&handle) {
            hide(&handle, false);
        }
    }
});
```
- `show`: keep `set_position` (no-op on Wayland), add `unminimize()` before `show()`/`set_focus()`, keep emitting `panel://shown`.
- `set_window_style`: theme only; vibrancy stays macOS-only.
- Surface: opaque, square corners (Tauri default). Rounded corners would need `transparent(true)` plus a compositor and are out of scope.

### 3. Wayland specifics
- Document that `always_on_top`, position and skip-taskbar are hints; enforce via WM rules (below).
- Autostart a resident instance so DBus toggles reach it. Documented, no new crate:
  - `~/.config/systemd/user/slopchat.service` → `ExecStart=/usr/bin/slopchat show`, `Restart=on-failure`; or
  - `~/.config/autostart/slopchat.desktop`.
- Soften the global-hotkey failure toast on Linux: only surface it on X11, so Wayland users aren't nagged about a shortcut they don't need.

### 4. Build and packaging
- `bundle.targets: "all"` already yields deb/rpm/AppImage on Linux; document the system dependencies and the keyring/Secret-Service requirement.
- Confirm the tray degrades gracefully when `libayatana-appindicator` is absent.

### 5. README
New `README.md`: overview, dev/build/test commands, Linux install/run, and:
- **Wayland:** bind a compositor keybind to `slopchat toggle`; autostart snippet; note the hints caveat.
- **WM float rules** (discover the id via `hyprctl clients` / `swaymsg -t get_tree` / `xprop WM_CLASS`):
  - Hyprland: `windowrulev2 = float,class:^(slopchat)$` (plus `noanim`, `center`)
  - sway: `for_window [app_id="slopchat"] floating enable, border none, resize set 680 480`
  - i3: `for_window [class="Slopchat"] floating enable, border none`
- Model guidance (AGENT_TUNING_PLAN §7): flash-class models are weak at multi-step tool use; suggest a `research` profile.

### 6. Verification
Linux cannot be built on macOS, so add continuous coverage:
- `.github/workflows/linux.yml` (`ubuntu-latest`): apt deps → `cargo clippy -- -D warnings` → `cargo test` → `npm ci && npm run check && npm test` → `cargo build` (or `cargo tauri build`).
- Manual acceptance still requires a real X11 (hotkey) and Wayland (CLI) session.

## Files
- New: `src-tauri/src/cli.rs`, `README.md`, `.github/workflows/linux.yml`.
- Changed: `src-tauri/src/main.rs`, `src-tauri/src/panel/linux.rs`, possibly `src-tauri/src/config/mod.rs` (Wayland toast), `src-tauri/tauri.conf.json` (metadata).

## Decisions to confirm
1. CLI scope: only `toggle`, or also `show`/`hide`/`quit`?
2. Autostart: documented systemd unit + `.desktop` (no dep) or the `autostart` plugin?
3. Surface: opaque square window (planned) or transparent with rounded corners?
4. Verification: CI job, Docker check, or manual only?
5. Wayland hotkey error: suppress, or keep the toast?
