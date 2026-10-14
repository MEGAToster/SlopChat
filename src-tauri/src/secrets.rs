use keyring::Entry;

const SERVICE: &str = "SlopChat";

// The freedesktop Secret Service store blocks on a runtime to talk to the
// daemon, and `block_on` panics when a tokio runtime already drives the current
// thread. Every keyring touchpoint can arrive from an async command or the
// streaming/search agent, so run the Entry work on a dedicated OS thread that
// has no runtime context. Joining blocks the worker only for the short local
// Secret Service call, which already blocks when it has to raise an unlock
// prompt.
fn offload<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T, String> {
    match std::thread::spawn(f).join() {
        Ok(value) => Ok(value),
        Err(_) => Err("keyring operation panicked".into()),
    }
}

pub fn get(account: &str) -> Result<Option<String>, String> {
    let account = account.to_string();
    offload(move || {
        let entry = Entry::new(SERVICE, &account).map_err(|error| {
            eprintln!("[keyring] new({account}) failed: {error}");
            error.to_string()
        })?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => {
                eprintln!("[keyring] get({account}) failed: {error}");
                Err(error.to_string())
            }
        }
    })?
}

pub fn set(account: &str, password: &str) -> Result<(), String> {
    let (account, password) = (account.to_string(), password.to_string());
    offload(move || {
        let entry = Entry::new(SERVICE, &account).map_err(|error| {
            eprintln!("[keyring] new({account}) failed: {error}");
            error.to_string()
        })?;
        entry.set_password(&password).map_err(|error| {
            eprintln!("[keyring] set({account}) failed: {error}");
            error.to_string()
        })
    })?
}

pub fn delete(account: &str) -> Result<(), String> {
    let account = account.to_string();
    offload(move || {
        let entry = Entry::new(SERVICE, &account).map_err(|error| error.to_string())?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    })?
}