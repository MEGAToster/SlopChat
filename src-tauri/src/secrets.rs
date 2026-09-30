use keyring::Entry;

const SERVICE: &str = "SlopChat";

pub fn get(account: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE, account).map_err(|error| error.to_string())?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

pub fn set(account: &str, password: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, account).map_err(|error| error.to_string())?;
    entry
        .set_password(password)
        .map_err(|error| error.to_string())
}

pub fn delete(account: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, account).map_err(|error| error.to_string())?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
