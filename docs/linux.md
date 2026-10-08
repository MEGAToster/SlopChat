# Linux

How the Linux build runs: the resident CLI, the Wayland layer surface, and the
platform fallbacks. API-key storage is in [[secrets]].

## Support matrix

| Environment | Behaviour |
| --- | --- |
| Wayland with `wlr-layer-shell` (Hyprland, sway, KDE Plasma, Mir) | Overlay layer surface, transparent, keybind via the CLI |
| GNOME on Wayland | No `wlr-layer-shell`; falls back to a normal toplevel |
| X11 | Normal toplevel; global shortcut via `XGrabKey` |

## Process model

The binary has no subcommand model in the usual sense — running `slopchat` starts a
resident, hidden instance and prints nothing. Later invocations are handed to the
running instance over D-Bus and exit. Mapping is in [`src-tauri/src/cli.rs`](../src-tauri/src/cli.rs):

- `slopchat` — start the resident instance (hidden).
- `slopchat toggle` / `show` / `hide` — control it.
- `slopchat quit` — stop it.

The single-instance name is `org.com_slopchat_app.SingleInstance` (derived from the bundle
identifier). The first instance owns it; later launches message it and exit. A
cold-started `toggle` shows the panel via `.setup` in
[`src-tauri/src/main.rs`](../src-tauri/src/main.rs).

## Layer surface (Wayland)

On Wayland the panel is a `wlr-layer-shell` **overlay** surface so the compositor does
not tile it. Setup lives in [`src-tauri/src/panel/linux.rs`](../src-tauri/src/panel/linux.rs)
(`init_layer_shell`, `apply_layer_geometry`).

Why `gtk-layer-shell` is required: GTK owns the `wl_surface` and fixes its role to
`xdg_toplevel` at first commit, so the layer-shell protocol cannot be attached directly.
`gtk-layer-shell` hooks GDK's `realize` and uses GTK-private APIs (`gtk-priv/`) to swap in
the layer-surface role before the first commit; a window that is already realized falls back
to XDG. The wrapper crate is marked unmaintained, but the C library is in maintenance mode
and still updated for GTK3.

Geometry details:

- **Layer** `Overlay`, namespace `slopchat`, exclusive zone `-1` (no reserved space).
- **Size**: layer surfaces ignore `set_position`, and on Wayland `gtk_window_resize` is a
  no-op. `apply_layer_geometry` drives the surface with `gtk_widget_set_size_request`
  (the surface follows the widget allocation) and positions with a top margin from the
  active monitor's work area — 18% for compact, vertically centred when expanded.
- **Keyboard**: `KeyboardMode::Exclusive`, so the composer receives keys while the panel
  is shown. While a native dialog (the file chooser) is open the surface drops to the
  bottom layer with keyboard mode `None` so the dialog can take focus, then restores.
- **Fallback**: `gtk_layer_is_supported()` is false on GNOME-on-Wayland and X11, so the
  panel stays a normal toplevel there. Use a float rule (see below).

## Transparency and startup

The Wayland window is built transparent so the theme's background opacity composites over
the compositor's backdrop (Hyprland can blur it; see the README `layer_rule`). Linux has no
native vibrancy. On X11 the window is opaque with a black background. To avoid a white
webview flash before first paint, [`index.html`](../index.html) starts black and
[`src/main.ts`](../src/main.ts) clears it to transparent once the panel is painted.

## Tray

The tray needs `libayatana-appindicator` at runtime. `platform.rs::tray_available`
probes the loader (`ldconfig -p`, then standard lib dirs and `LD_LIBRARY_PATH`) before
building it, so a missing library does not crash the app — it just runs without a tray
entry, and `slopchat quit` stops it. The library's own deprecation notice is filtered in
`platform.rs::silence_appindicator_deprecation`.

## Global shortcuts

`tauri-plugin-global-shortcut` uses X11 `XGrabKey`, so the configured toggle works only on
X11. On Wayland, bind a compositor keybind to `slopchat toggle` instead. A failed
registration is not surfaced on Wayland (`platform::is_wayland`), since it is expected.

## Input quirks

- WebKitGTK does not bridge the system clipboard into the DOM `clipboardData`, so paste
  is read through a native command (`src-tauri/src/clipboard.rs`, `arboard`) when the
  paste event carries no text. Text is inserted at the caret; a pasted image becomes an
  attachment.
- WebKitGTK reports Shift+Tab as `event.key === "Unidentified"`; `keymap/parse.ts` falls
  back to `event.code`/`keyCode` so the profile keybind still matches.
- Native `<select>` controls pick up the GTK theme; `styles/base.css` disables the native
  rendering for `select.control` so they follow the app theme.

## Build and install

`scripts/install.sh` builds from source and installs `~/.local/bin/slopchat` plus a desktop
entry. Fedora dependencies: `webkit2gtk4.1-devel`, `libsoup3-devel`, `librsvg2-devel`,
`libayatana-appindicator-gtk3-devel`, `gtk-layer-shell-devel`, `patchelf`. Runtime:
`libayatana-appindicator-gtk3` (tray) and `gtk-layer-shell` (overlay layer).

[`.github/workflows/linux.yml`](../.github/workflows/linux.yml) runs clippy, tests and a
build on `ubuntu-latest`.

## Hyprland

The README shows the `exec`/`bind`/`layer_rule` snippet. For non-wlroots compositors, float
the window with a rule instead: sway `for_window [app_id="slopchat"] floating enable, border
none`; i3 `for_window [class="Slopchat"] floating enable, border none`.

## Code map

- [`src-tauri/src/panel/linux.rs`](../src-tauri/src/panel/linux.rs) — window, layer surface, geometry.
- [`src-tauri/src/clipboard.rs`](../src-tauri/src/clipboard.rs) — native clipboard read for paste.
- [`src-tauri/src/platform.rs`](../src-tauri/src/platform.rs) — Wayland detection, tray probe, log filter.
- [`src-tauri/src/cli.rs`](../src-tauri/src/cli.rs) — CLI verbs.
- [`scripts/install.sh`](../scripts/install.sh) — Linux build/install.
