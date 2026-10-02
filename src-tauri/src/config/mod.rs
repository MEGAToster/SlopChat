mod defaults;
mod keybinds;
mod models_file;
mod settings;
mod themes;
mod watch;
mod write;

pub use settings::*;
pub use themes::UserTheme;
pub(crate) use settings::migrate_profiles;

use std::sync::{Mutex, RwLock};

use tauri::{AppHandle, Emitter, Manager};

pub struct ConfigState {
    settings: RwLock<Settings>,
    error: Mutex<Option<String>>,
}

impl ConfigState {
    fn new(settings: Settings) -> Self {
        Self {
            settings: RwLock::new(settings),
            error: Mutex::new(None),
        }
    }

    pub fn get(&self) -> Settings {
        self.settings.read().expect("config lock poisoned").clone()
    }

    pub fn error(&self) -> Option<String> {
        self.error.lock().expect("config lock poisoned").clone()
    }

    fn set(&self, settings: Settings) {
        *self.settings.write().expect("config lock poisoned") = settings;
    }

    fn set_error(&self, error: Option<String>) {
        *self.error.lock().expect("config lock poisoned") = error;
    }
}

pub fn init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let (settings, load_error) = match write::load() {
        Ok(settings) => (settings, None),
        Err(error) => (Settings::default(), Some(error)),
    };

    app.manage(ConfigState::new(settings.clone()));

    if let Some(error) = load_error {
        report_error(app, error);
    }

    apply_toggle(app, &settings.keybinds.toggle);
    watch::start(app.clone())?;
    Ok(())
}

pub(crate) fn reload(app: &AppHandle) {
    let settings = match write::load() {
        Ok(settings) => settings,
        Err(error) => {
            report_error(app, error);
            return;
        }
    };

    let shortcuts = match keybinds::parse(&settings.keybinds.toggle) {
        Ok(shortcuts) => shortcuts,
        Err(error) => {
            report_error(app, error);
            return;
        }
    };

    let state = app.state::<ConfigState>();
    let toggle_changed = state.get().keybinds.toggle != settings.keybinds.toggle;

    if toggle_changed {
        if let Err(error) = keybinds::apply(app, &shortcuts) {
            // Wayland compositors handle keybinds themselves; a failed X11 grab
            // is expected and should not block the settings update.
            if !crate::platform::is_wayland() {
                report_error(app, error);
                return;
            }
        }
    }

    state.set(settings.clone());
    state.set_error(None);
    let _ = app.emit("config://changed", &settings);
}

pub fn ensure_file() -> Result<(), String> {
    write::ensure_file()
}

pub fn save_model(
    app: &AppHandle,
    config: &ModelConfig,
    previous: Option<&str>,
) -> Result<(), String> {
    write::save_model(app, config, previous)
}

pub fn remove_model(app: &AppHandle, model: &str) -> Result<(), String> {
    write::remove_model(app, model)
}

pub fn save_profile(app: &AppHandle, profile: &Profile) -> Result<(), String> {
    write::save_profile(app, profile)
}

pub fn remove_profile(app: &AppHandle, name: &str) -> Result<(), String> {
    write::remove_profile(app, name)
}

pub fn set_theme(app: &AppHandle, theme: &str) -> Result<(), String> {
    write::set_theme(app, theme)
}

pub fn set_style(app: &AppHandle, style: &str) -> Result<(), String> {
    write::set_style(app, style)
}

pub fn save_advanced(app: &AppHandle, advanced: &Advanced) -> Result<(), String> {
    write::save_advanced(app, advanced)
}

pub fn set_startup(app: &AppHandle, startup: Startup) -> Result<(), String> {
    write::set_startup(app, startup)
}

pub fn set_resume_within(app: &AppHandle, minutes: u64) -> Result<(), String> {
    write::set_resume_within(app, minutes)
}

pub fn set_reader_fallback(app: &AppHandle, enabled: bool) -> Result<(), String> {
    write::set_reader_fallback(app, enabled)
}

pub fn set_agent_log(app: &AppHandle, enabled: bool) -> Result<(), String> {
    write::set_agent_log(app, enabled)
}

pub fn save_window(app: &AppHandle, window: &WindowConfig) -> Result<(), String> {
    write::save_window(app, window)
}

pub fn load_user_themes() -> Vec<UserTheme> {
    themes::load_user()
}

pub fn open_file() -> Result<(), String> {
    ensure_file()?;
    open_in_editor(&crate::paths::settings_path())
}

pub fn open_models_file() -> Result<(), String> {
    ensure_file()?;
    open_in_editor(&crate::paths::models_path())
}

fn open_in_editor(path: &std::path::Path) -> Result<(), String> {
    if let Some(editor) = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .ok()
        .filter(|editor| !editor.trim().is_empty())
    {
        return std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("exec {editor} \"$1\""))
            .arg("sh")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string());
    }

    crate::platform::open_path(path)
}

fn apply_toggle(app: &AppHandle, keys: &[String]) {
    let shortcuts = match keybinds::parse(keys) {
        Ok(shortcuts) => shortcuts,
        Err(error) => {
            report_error(app, error);
            return;
        }
    };
    if let Err(error) = keybinds::apply(app, &shortcuts) {
        if !crate::platform::is_wayland() {
            report_error(app, error);
        }
    }
}

fn report_error(app: &AppHandle, error: String) {
    let message = format!("Config not applied, still using previous settings — {error}");
    app.state::<ConfigState>().set_error(Some(message.clone()));
    let _ = app.emit("config://error", &message);
}
