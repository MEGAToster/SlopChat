# SlopChat — Linux support

Shipped for Wayland, with Hyprland/Fedora as the reference setup. macOS remains the reference platform.

## Commands
`src-tauri/src/cli.rs` maps argv to actions; the single-instance callback in `main.rs` routes later launches to the resident instance, and `.setup` handles the cold-start case.

- `slopchat` (no arguments) — resident instance, hidden.
- `slopchat toggle` / `show` / `hide` — control the running instance.
- `slopchat quit` — stop it.

## Layer shell (Wayland)
On Wayland the panel is a `wlr-layer-shell` **overlay** surface (via `gtk-layer-shell`), not a normal toplevel. Compositors therefore don't tile it, and layer rules (e.g. Hyprland blur) apply. `panel/linux.rs` initialises it after the window is built but before it is mapped: `is_supported()` → `init_layer_shell`, namespace `slopchat`, `Layer::Overlay`, keyboard mode `Exclusive`, exclusive zone `-1`, anchor `Top`.

**Why `gtk-layer-shell` is required:** GTK owns the `wl_surface` and fixes its role to `xdg_toplevel` at first commit, so the `wlr-layer-shell` protocol cannot be attached directly from a GTK/WebKit window. `gtk-layer-shell` hooks GDK's `realize` and uses GTK-private APIs to swap in the layer-surface role; if the window is already realized it silently falls back to XDG. The Rust wrapper crate is marked unmaintained, but the C library is in maintenance mode and still updated for GTK3.

**Sizing/positioning:** layer surfaces ignore `set_position` and, on Wayland, `gtk_window_resize`. `panel/linux.rs` sets the size with `gtk_widget_set_size_request` (the surface follows the widget allocation) and positions with a top margin from the active monitor's work area — 18% for compact, vertically centred when expanded.

**Fallback:** `gtk_layer_is_supported()` is false on GNOME-on-Wayland and X11, so the panel stays a normal toplevel there; float it with a compositor rule.

## Hyprland
```conf
exec = slopchat
bind = SUPER, SPACE, exec, slopchat toggle
```
`exec` re-runs on reload; the start is idempotent because of single-instance. For blur/animation, add a layer rule matching `namespace = slopchat`.

## API keys
`keyring` v4 on Linux uses a Secret Service (GNOME Keyring / KWallet via DBus); there is no keyutils fallback. A bare Hyprland session may have none, so reference an environment variable in `settings.json` and export it:
```json
{ "providers": { "openrouter": { "baseUrl": "https://openrouter.ai/api/v1", "apiKey": "env:OPENROUTER_API_KEY" } } }
```

## Build
`scripts/install.sh` builds from source on Linux and installs `~/.local/bin/slopchat` plus a desktop entry. Fedora dependencies: `webkit2gtk4.1-devel`, `libsoup3-devel`, `librsvg2-devel`, `libayatana-appindicator-gtk3-devel`, `gtk-layer-shell-devel`, `patchelf`. Runtime: `libayatana-appindicator-gtk3` (tray) and `gtk-layer-shell` (overlay layer).

## Platform findings
- **Single instance uses DBus**, not a lockfile: the first instance owns `org.com_slopchat_app.SingleInstance`; later launches message it and exit. Snap/Flatpak need `--talk-name`/`--own-name` for that service.
- **`tauri-plugin-global-shortcut` does not work on Wayland** (X11 `XGrabKey`), so the CLI is the Wayland path. A failed registration is not surfaced on Wayland (`platform::is_wayland`).
- **No hide-on-blur on Linux**; focus loss is not used (removed for Hyprland). `panel::is_suppressed` is therefore macOS-only.
- **Tray** needs `libayatana-appindicator`; if absent the app logs `tray unavailable` and keeps running. The library's deprecation notice is filtered via `platform::silence_appindicator_deprecation`.
- **CI**: `.github/workflows/linux.yml` runs clippy, tests and a build on `ubuntu-latest`.