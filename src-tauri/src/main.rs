#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod commands;
mod config;
mod panel;
mod paths;
mod platform;
mod providers;
mod secrets;
mod store;
mod tools;

use crate::providers::client::Streams;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

fn main() {
    #[cfg(target_os = "linux")]
    platform::silence_appindicator_deprecation();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            cli::run(app, &argv);
        }))
        .manage(panel::Sticky::default())
        .manage(panel::PanelState::default())
        .manage(Streams::default());

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
            commands::get_settings,
            commands::config_error,
            commands::resize_panel,
            commands::set_panel_expanded,
            commands::toggle_panel_expanded,
            commands::is_panel_expanded,
            commands::set_window_style,
            commands::has_api_key,
            commands::store_api_key,
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
            commands::append_message,
            commands::rewind_to,
            commands::search_chats,
            commands::set_suppress_hide,
            commands::save_attachment,
            commands::read_attachment,
            commands::list_models,
            commands::open_settings_file,
            commands::reload_config,
            commands::save_model,
            commands::remove_model,
            commands::save_profile,
            commands::remove_profile,
            commands::model_reasoning,
            commands::set_theme,
            commands::set_style,
            commands::save_advanced,
            commands::set_startup,
            commands::set_resume_within,
            commands::set_reader_fallback,
            commands::set_agent_log,
            commands::open_agent_log,
            commands::save_window,
            commands::load_user_themes,
            commands::open_url,
            commands::search_usage,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            config::init(app.handle())?;
            panel::init(app.handle())?;
            if platform::tray_available() {
                if let Err(error) = setup_tray(app.handle()) {
                    eprintln!("tray unavailable: {error}");
                }
            } else {
                eprintln!("tray unavailable: libayatana-appindicator3 not found");
            }

            let store = store::Store::open(&paths::chats_db_path()).unwrap_or_else(|error| {
                eprintln!("history unavailable, using in-memory store: {error}");
                store::Store::in_memory().expect("in-memory store")
            });
            app.manage(store);

            let argv: Vec<String> = std::env::args().collect();
            let handle = app.handle().clone();
            let _ = app.run_on_main_thread(move || cli::cold_start(&handle, &argv));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running SlopChat");
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
