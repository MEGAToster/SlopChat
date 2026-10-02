use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Position, Size, WebviewUrl, WebviewWindowBuilder,
};

use super::{DEFAULT_COMPACT_WIDTH, MIN_HEIGHT};

const LABEL: &str = "main-panel";

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("SlopChat")
        .inner_size(DEFAULT_COMPACT_WIDTH, MIN_HEIGHT)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        // Black behind the webview avoids a white flash before first paint on
        // X11; on Wayland the surface is transparent for compositor effects.
        .background_color(tauri::window::Color(0, 0, 0, 255))
        .transparent(crate::platform::is_wayland())
        .visible(false)
        .build()?;

    #[cfg(target_os = "linux")]
    {
        init_layer_shell(&window);
        apply_layer_geometry(app, &window, DEFAULT_COMPACT_WIDTH, MIN_HEIGHT);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn init_layer_shell(window: &tauri::WebviewWindow) {
    use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

    let Ok(gtk_window) = window.gtk_window() else {
        return;
    };
    if !gtk_layer_shell::is_supported() {
        return;
    }

    gtk_window.init_layer_shell();
    gtk_window.set_namespace("slopchat");
    gtk_window.set_layer(Layer::Overlay);
    gtk_window.set_keyboard_mode(KeyboardMode::Exclusive);
    gtk_window.set_exclusive_zone(-1);
    gtk_window.set_anchor(Edge::Top, true);
}

// Layer surfaces are placed by the compositor, so position is expressed as a
// top margin. Returns false when the window is not a layer surface.
#[cfg(target_os = "linux")]
fn apply_layer_geometry(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    width: f64,
    height: f64,
) -> bool {
    use gtk::prelude::WidgetExt;
    use gtk_layer_shell::{Edge, LayerShell};

    let Ok(gtk_window) = window.gtk_window() else {
        return false;
    };
    if !gtk_window.is_layer_window() {
        return false;
    }

    let (_, _, _, area_height) = super::work_area(app);
    let margin = if super::is_expanded(app) {
        (area_height - height) / 2.0
    } else {
        area_height * 0.18
    };
    gtk_window.set_layer_shell_margin(Edge::Top, margin.max(0.0).round() as i32);
    // gtk_window_resize is ignored on Wayland; the layer surface follows the
    // widget allocation, so a size request is what actually resizes it.
    gtk_window.set_size_request(width as i32, height as i32);
    true
}

pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        if !super::is_expanded(app) {
            let _ = window.set_position(super::compact_position(app));
        }
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("panel://shown", ());
    }
}

pub fn hide(app: &AppHandle, _restore: bool) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.hide();
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window(LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

pub fn set_window_style(app: &AppHandle, scheme: &str, _vibrancy: &str) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let theme = if scheme == "light" {
            tauri::Theme::Light
        } else {
            tauri::Theme::Dark
        };
        let _ = window.set_theme(Some(theme));
    }
}

pub fn resize(app: &AppHandle, position: Position, width: f64, height: f64) {
    if let Some(window) = app.get_webview_window(LABEL) {
        #[cfg(target_os = "linux")]
        if apply_layer_geometry(app, &window, width, height) {
            return;
        }
        let _ = window.set_position(position);
        let _ = window.set_size(Size::Logical(LogicalSize::new(width, height)));
    }
}
