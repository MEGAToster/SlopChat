# Research plan: Linux clipboard and layer stacking

Self-contained plan for whoever next has a Linux (Hyprland/Wayland) machine. Read `AGENTS.md` and
`docs/linux.md` first. Nothing here has been tested; every claim marked **(unverified)** must be
confirmed before any code depends on it.

Work in phases: **observe → isolate → experiment → implement**. Record the findings in the
**Results** section at the bottom, then turn the chosen fixes into code.

## Problems

1. **The clipboard manager's picker appears below the SlopChat panel and can't be used.** It is
   reportedly on the same layer.
2. **SlopChat may be interfering with the clipboard manager in general.** The timing matches
   commit `2b3477f`, which added `arboard` (with `wayland-data-control`) for native paste, but it
   isn't confirmed.

## Current behaviour (from the code)

- `src-tauri/src/panel/linux.rs` `init_layer_shell` sets up the panel as a layer surface:
  - `Layer::Overlay`, the highest layer.
  - `KeyboardMode::Exclusive`, which takes keyboard focus while the panel is shown, so no other
    surface can get keys.
  - Namespace `slopchat`.
- `apply_suppressed` moves the panel to `Layer::Bottom` with `KeyboardMode::None` while a native
  file dialog is open, for the same reason.
- `hide()` → `window.hide()` should unmap the surface, so it should release the keyboard grab
  when the panel is hidden **(unverified)**.
- `src-tauri/src/clipboard.rs` reads the clipboard with `arboard`:
  - It is read-only (`get_text`, then `get_image`).
  - It is only called from `Composer.svelte` `onPaste`, when WebKitGTK's `clipboardData` is
    empty.
  - It holds no state between calls and never takes ownership of the selection.
- Linux-only dependencies involved: `arboard` (which pulls in `wl-clipboard-rs`, `image` and an
  X11 fallback) and `png`, which is only used to re-encode clipboard images.

## Hypotheses

| ID | Hypothesis | Explains |
| --- | --- | --- |
| H1 | The picker is on the `top` layer, so it is always below SlopChat's `overlay`. | Problem 1 (stacking) |
| H2 | Both are on `overlay`, and Hyprland orders same-layer surfaces in a way that puts the picker below. | Problem 1 (stacking) |
| H3 | Even when the picker is visible, SlopChat's `Exclusive` keyboard grab stops the picker (and any auto-paste tool like `wtype`) from getting keys. | Problem 1 (unusable) and possibly 2 |
| H4 | The panel doesn't fully release the grab or unmap when hidden, so it affects the system even when not visible. | Problem 2 |
| H5 | `arboard`'s data-control read disturbs the clipboard manager (for example by triggering a re-store or a selection change). It only happens on paste into SlopChat. | Problem 2 |
| H6 | Problem 2 isn't caused by SlopChat at all, and the timing is a coincidence (a compositor, manager or package update). | Problem 2 |

## Phase 1 — Observe

Record exact versions: `hyprctl version`, the clipboard manager and its version, the picker and
its version, and `slopchat`'s commit.

1. **Describe the symptom precisely.** Which of these happens?
   - entries missing from history;
   - duplicates;
   - the clipboard emptied after something;
   - the picker not opening or not responding;
   - a picked entry not pasting.

   Note what you did right before it happens.
2. **Stacking.** Open the SlopChat panel, then the picker, and run:
   ```sh
   hyprctl layers
   ```
   Record each surface's namespace and its level (0 background, 1 bottom, 2 top, 3 overlay).
   - Picker on level 2 → **H1**.
   - Both on 3 → **H2**.
3. **Keyboard.** With both open and the picker visibly on top (move SlopChat away or shrink it if
   needed), type into the picker. If it gets no keys, that supports **H3**.
4. **Hidden state.** Hide the panel (`slopchat hide`) and run `hyprctl layers` again. If a
   `slopchat` entry is still listed, that supports **H4**.

## Phase 2 — Isolate problem 2

Do each step for long enough to trigger the symptom (or for a fixed time, e.g. 30 min of normal
use):

1. `slopchat quit` and use the clipboard manager normally. If the symptom still happens →
   **H6**. Stop investigating SlopChat for problem 2.
2. Start `slopchat`, never open the panel. If the symptom appears → **H4**, or something at
   startup.
3. Open and close the panel repeatedly, but never paste into it. If the symptom appears →
   **H3/H4**: it comes from the layer surface, not `arboard`.
4. Paste text and an image into SlopChat several times. If the symptom only appears now →
   **H5**.

For step 4, watch the manager live while pasting into SlopChat:
```sh
wl-paste --watch sh -c 'date +%T; wl-paste --list-types'
```
A new entry printed on paste means SlopChat is changing the selection (it shouldn't).

## Phase 3 — Experiments

Run these only for the hypotheses Phase 1–2 supported. Each one is a small local change, not a
commit.

### E1 — `KeyboardMode::OnDemand` (H3, H4)

In `init_layer_shell`, change `KeyboardMode::Exclusive` to `KeyboardMode::OnDemand` (the `v0_6`
feature is already enabled). Then check:

- Does `slopchat toggle` still focus the composer without a click? This is the key risk: it's
  unverified whether Hyprland focuses an on-demand layer surface when it maps.
- Can the picker take keyboard focus while the panel is open?
- Does clicking back into the panel give it focus again?
- Does the file chooser now work **without** `apply_suppressed`? Comment the call out in
  `Composer.svelte` `pick()` to test.
- After picking an entry, does `Ctrl+V` in the panel paste it?

If the composer doesn't get focus on show, try **E1b**:
- Set `Exclusive` in `show()`.
- Switch to `OnDemand` once the window reports focus. Use GTK `connect_focus_in_event` or the
  first `panel://shown` round-trip from the frontend.
- Check that switching keyboard mode while mapped doesn't drop focus.

### E2 — Stacking (H1, H2)

Try in order and keep the first that works:

1. **H1:** configure the picker for the overlay layer, for example fuzzel `layer=overlay`. Other
   pickers have their own option. No SlopChat change is needed.
2. **H2:** try a Hyprland layer rule that sets `order` for the picker namespace (or a negative
   `order` for `slopchat`). Confirm the key name and sign in the Hyprland wiki for your version,
   since the README uses the Lua `hl.layer_rule` syntax **(unverified)**.
3. Only if both fail: make SlopChat's layer configurable (`overlay` / `top`). Note the trade-off:
   on `top`, the panel won't show over fullscreen windows.

### E3 — GTK clipboard instead of `arboard` (H5, and a lighter build)

Replace `clipboard::read()` on Linux with GTK's clipboard. This uses the normal focused-client
paste path, not the data-control protocol that clipboard managers use.

- Get the clipboard with `gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD)`.
- Read text with `wait_for_text()`.
- For images, call `wait_for_image()`, which returns a `gdk_pixbuf::Pixbuf`, then
  `save_to_bufferv("png", &[])` to get PNG bytes, then base64.
- GTK must be called on the main thread. The `read_clipboard` command runs on an async worker,
  so use `app.run_on_main_thread` and return the result over a channel (`std::sync::mpsc` or a
  `tokio::sync::oneshot`). `wait_for_*` runs a nested main loop, so the UI won't freeze.

Check:

- Does pasting text work in the panel? This is the key risk: the original bug was WebKitGTK not
  filling `clipboardData`. It's unverified whether GTK's own clipboard API works on a layer
  surface.
- Does pasting an image (screenshot to clipboard) become an attachment?
- Does it still work with `KeyboardMode::OnDemand` from E1?
- Does the Phase 2 step 4 `wl-paste --watch` check stay quiet?

If it works, it removes `arboard`, `png` and the transitive `wl-clipboard-rs`, `image` and X11
code from the Linux build. Check the bundle size before and after.

## Phase 4 — Implement

Turn the experiments that worked into commits. Keep platform code in `panel/` and `clipboard.rs`
only.

1. Keyboard mode change (E1 or E1b). Delete `apply_suppressed`, the `set_suppress_hide` path and
   the `Composer.svelte` dialog workaround if E1 made them unnecessary.
2. The clipboard swap (E3), if it worked. Remove `arboard` and `png` from
   `[target.'cfg(target_os = "linux")'.dependencies]` in `Cargo.toml`.
3. A configurable layer (E2.3), only if it was needed.
4. Docs:
   - `docs/linux.md`: update the "Keyboard" bullet, the "Input quirks" clipboard bullet and the
     code map.
   - README: Hyprland snippet if a layer rule is recommended.
   - Add a short "clipboard managers" note with the picker layer advice.
5. Checks: `cargo test`, `cargo clippy -- -D warnings`, `npm test`, `npm run check`, and the
   Linux CI workflow (`.github/workflows/linux.yml`).

## Results

Fill in during Phase 1–3.

- Versions:
- Symptom description:
- `hyprctl layers` (panel open, picker open, panel hidden):
- Phase 2 outcome (which step first showed the symptom):
- E1 / E1b:
- E2:
- E3:
- Decision:
