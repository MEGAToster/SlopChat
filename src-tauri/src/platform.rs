use std::path::Path;

pub fn open_path(target: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(not(target_os = "macos"))]
    let opener = "xdg-open";

    std::process::Command::new(opener)
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn is_wayland() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "wayland")
}

// libappindicator is deprecated in favour of a GLib variant we cannot use yet;
// drop only that domain so the notice doesn't spam stderr on every launch.
#[cfg(target_os = "linux")]
pub fn silence_appindicator_deprecation() {
    glib::log_set_default_handler(|domain, level, message| {
        if domain != Some("libayatana-appindicator") {
            glib::log_default_handler(domain, level, Some(message));
        }
    });
}

#[cfg(not(target_os = "linux"))]
pub fn tray_available() -> bool {
    true
}

// libappindicator panics if the dynamic library is missing, so probe for it
// before letting Tauri build the tray.
#[cfg(target_os = "linux")]
pub fn tray_available() -> bool {
    let listed = ["ldconfig", "/sbin/ldconfig", "/usr/sbin/ldconfig"]
        .iter()
        .find_map(|command| std::process::Command::new(command).arg("-p").output().ok());
    if let Some(output) = listed {
        if cache_lists_tray(&String::from_utf8_lossy(&output.stdout)) {
            return true;
        }
    }

    let mut dirs: Vec<std::path::PathBuf> = std::env::var_os("LD_LIBRARY_PATH")
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();
    for dir in [
        "/usr/lib64",
        "/usr/lib",
        "/usr/lib/x86_64-linux-gnu",
        "/lib64",
        "/lib",
    ] {
        dirs.push(std::path::PathBuf::from(dir));
    }

    dirs.iter()
        .any(|dir| TRAY_LIBS.iter().any(|name| dir.join(name).exists()))
}

#[cfg(target_os = "linux")]
const TRAY_LIBS: [&str; 4] = [
    "libayatana-appindicator3.so.1",
    "libappindicator3.so.1",
    "libayatana-appindicator3.so",
    "libappindicator3.so",
];

#[cfg(target_os = "linux")]
fn cache_lists_tray(cache: &str) -> bool {
    TRAY_LIBS.iter().any(|name| cache.contains(name))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::cache_lists_tray;

    #[test]
    fn detects_the_tray_library_in_the_loader_cache() {
        assert!(cache_lists_tray(
            "libayatana-appindicator3.so.1 (libc6,x86-64) => /lib64/libayatana-appindicator3.so.1"
        ));
        assert!(cache_lists_tray("libappindicator3.so.1 => /usr/lib64/libappindicator3.so.1"));
        assert!(!cache_lists_tray("libzzipwrap-0.so.13 => /lib64/libzzipwrap-0.so.13"));
        assert!(!cache_lists_tray(""));
    }
}
