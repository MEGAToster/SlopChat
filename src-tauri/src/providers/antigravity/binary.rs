use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;

use super::exec::AgyError;

// 1.1.15 is where `--input-format stream-json` appeared; 1.2.16 fixed headless
// runs exiting before the agent answered.
const MIN_VERSION: Version = Version(1, 1, 15);
const RECOMMENDED: Version = Version(1, 2, 16);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Version(pub u32, pub u32, pub u32);

impl std::fmt::Display for Version {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.0, self.1, self.2)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub path: Option<String>,
    pub version: Option<String>,
    pub ok: bool,
    pub note: Option<String>,
}

fn cache() -> &'static Mutex<Option<(PathBuf, Version)>> {
    static CACHE: OnceLock<Mutex<Option<(PathBuf, Version)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn override_path() -> &'static Mutex<Option<PathBuf>> {
    static OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
pub fn set_override_for_tests(path: Option<PathBuf>) {
    *override_path().lock().expect("override lock") = path;
    invalidate();
}

fn locate() -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(path) = override_path().lock().expect("override lock").clone() {
        return Some(path);
    }

    // A Finder-launched macOS app has no shell PATH, so probe the usual spot first.
    if let Some(home) = std::env::var_os("HOME") {
        let candidate = PathBuf::from(home).join(".local/bin").join("agy");
        if executable(&candidate) {
            return Some(candidate);
        }
    }

    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("agy"))
        .find(|candidate| executable(candidate))
}

fn executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|meta| meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

pub async fn version(path: &Path) -> Result<Version, String> {
    let output = tokio::time::timeout(
        Duration::from_secs(2),
        tokio::process::Command::new(path).arg("--version").output(),
    )
    .await
    .map_err(|_| "agy --version timed out".to_string())?
    .map_err(|error| error.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    parse_version(&text).ok_or_else(|| "could not parse agy version".to_string())
}

fn parse_version(text: &str) -> Option<Version> {
    for token in text.split(|c: char| !(c.is_ascii_digit() || c == '.')) {
        let mut parts = token.split('.');
        if let (Some(major), Some(minor), Some(patch)) = (parts.next(), parts.next(), parts.next()) {
            if let (Ok(major), Ok(minor), Ok(patch)) =
                (major.parse(), minor.parse(), patch.parse())
            {
                return Some(Version(major, minor, patch));
            }
        }
    }
    None
}

// A successful check is cached; `invalidate` clears it when a spawn later fails.
pub async fn check() -> Result<PathBuf, AgyError> {
    if let Some((path, _)) = cache().lock().expect("cache lock").clone() {
        return Ok(path);
    }

    let path = locate().ok_or(AgyError::NotInstalled)?;
    let found = version(&path).await.map_err(AgyError::Failed)?;
    if found < MIN_VERSION {
        return Err(AgyError::TooOld(found.to_string()));
    }

    *cache().lock().expect("cache lock") = Some((path.clone(), found));
    Ok(path)
}

pub fn invalidate() {
    *cache().lock().expect("cache lock") = None;
}

pub async fn status() -> Status {
    let Some(path) = locate() else {
        return Status {
            path: None,
            version: None,
            ok: false,
            note: None,
        };
    };

    match version(&path).await {
        Ok(found) => Status {
            path: Some(path.display().to_string()),
            version: Some(found.to_string()),
            ok: found >= MIN_VERSION,
            note: (found < RECOMMENDED).then(|| "update recommended (≥ 1.2.16)".to_string()),
        },
        Err(error) => Status {
            path: Some(path.display().to_string()),
            version: None,
            ok: false,
            note: Some(error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_version_line() {
        assert_eq!(parse_version("1.2.16"), Some(Version(1, 2, 16)));
        assert_eq!(parse_version("agy version v1.1.5\n"), Some(Version(1, 1, 5)));
        assert_eq!(parse_version("no digits here"), None);
    }

    #[test]
    fn version_ordering_compares_numerically() {
        assert!(Version(1, 2, 16) > Version(1, 1, 15));
        assert!(Version(1, 10, 0) > Version(1, 9, 9));
    }
}
