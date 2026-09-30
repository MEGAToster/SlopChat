#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod panel;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const TOGGLE_SHORTCUT: &str = "Alt+Space";

fn main() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            panel::show(app);
        }))
        .manage(panel::Sticky::default());

    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        panel::toggle(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::show_panel,
            commands::hide_panel,
            commands::toggle_panel,
            commands::toggle_sticky,
            commands::is_sticky,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            panel::init(app.handle())?;
            setup_tray(app.handle())?;
            register_toggle_shortcut(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running SlopChat");
}

fn register_toggle_shortcut(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut: Shortcut = TOGGLE_SHORTCUT.parse()?;
    app.global_shortcut().register(shortcut)?;
    Ok(())
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Show SlopChat", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit SlopChat", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => panel::show(app),
            "quit" => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app)?;
    Ok(())
}
