#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod clipboard;
mod commands;
mod config;
mod panel;
mod paths;
mod platform;
mod providers;
mod secrets;
mod store;
mod text;
mod tools;

use crate::providers::client::Streams;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_global_shortcut::ShortcutState;

fn main() {
    platform::silence_appindicator_deprecation();

    let (store, store_error) = open_store();
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            cli::run(app, &argv);
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(panel::Sticky::default())
        .manage(panel::PanelState::default())
        .manage(Streams::default())
        .manage(store);

    let builder = platform::attach_panel(builder);

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
            commands::hide_panel,
            commands::toggle_sticky,
            commands::is_sticky,
            commands::get_settings,
            commands::config_error,
            commands::resize_panel,
            commands::resize_overlay,
            commands::set_panel_expanded,
            commands::set_window_style,
            commands::has_api_key,
            commands::store_api_key,
            commands::verify_api_key,
            commands::delete_api_key,
            commands::send,
            commands::stop,
            commands::initial_chat,
            commands::list_chats,
            commands::create_chat,
            commands::get_chat,
            commands::delete_chat,
            commands::toggle_chat_pin,
            commands::set_chat_profile,
            commands::rename_chat,
            commands::set_chat_model,
            commands::append_message,
            commands::rewind_to,
            commands::search_chats,
            commands::set_suppress_hide,
            commands::platform_info,
            commands::read_clipboard,
            commands::import_paths,
            commands::save_attachment,
            commands::read_attachment,
            commands::list_models,
            commands::antigravity_status,
            commands::open_settings_file,
            commands::open_models_file,
            commands::reload_config,
            commands::save_model,
            commands::remove_model,
            commands::save_profile,
            commands::remove_profile,
            commands::model_reasoning,
            commands::set_theme,
            commands::set_style,
            commands::set_reading_width,
            commands::save_advanced,
            commands::set_startup,
            commands::set_resume_within,
            commands::set_paste_as_file,
            commands::set_reader_fallback,
            commands::set_searxng_url,
            commands::set_agent_log,
            commands::open_agent_log,
            commands::save_window,
            commands::save_font,
            commands::load_user_themes,
            commands::open_url,
            commands::search_usage,
        ])
        .setup(move |app| {
            platform::set_activation_policy(app);

            config::init(app.handle())?;
            if let Some(error) = store_error {
                config::warn(app.handle(), error);
            }
            panel::init(app.handle())?;
            if platform::tray_available() {
                if let Err(error) = setup_tray(app.handle()) {
                    eprintln!("tray unavailable: {error}");
                }
            } else {
                eprintln!("tray unavailable: libayatana-appindicator3 not found");
            }

            let argv: Vec<String> = std::env::args().collect();
            let handle = app.handle().clone();
            let _ = app.run_on_main_thread(move || cli::cold_start(&handle, &argv));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running SlopChat");
}

// Opens history before the window exists so an early IPC call cannot race it.
fn open_store() -> (store::Store, Option<String>) {
    match store::Store::open(&paths::chats_db_path()) {
        Ok(store) => (store, None),
        Err(error) => {
            eprintln!("history unavailable, using in-memory store: {error}");
            (
                store::Store::in_memory().expect("in-memory store"),
                Some(format!("History is unavailable and chats will not be saved — {error}")),
            )
        }
    }
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
