//! Account tokens in the app's private Library, outside the Documents folder
//! the Files app shows. iOS Data Protection encrypts them while locked.
//! A Keychain store replaces this once the app is signed with an entitlement.
use crate::account::SecretStore;
use std::{path::PathBuf, sync::Mutex};
static LOCK: Mutex<()> = Mutex::new(());
pub struct AndroidStore;
fn path(key: &str) -> Result<PathBuf, String> {
    if key.is_empty()
        || !key
            .strip_prefix("ely-")
            .unwrap_or(key)
            .bytes()
            .all(|c| c.is_ascii_hexdigit() || c == b'-')
    {
        return Err("Invalid account key".into());
    }
    let root = std::env::var_os("HOME").ok_or("No app sandbox")?;
    Ok(PathBuf::from(root)
        .join("Library/Application Support/Horizon/account-secrets")
        .join(format!("{key}.secret")))
}
impl SecretStore for AndroidStore {
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        let _guard = LOCK.lock().map_err(|_| "Account storage unavailable")?;
        match std::fs::read_to_string(path(key)?) {
            Ok(text) => Ok(Some(text)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err("Cannot read saved account".into()),
        }
    }
    fn set(&self, key: &str, secret: &str) -> Result<(), String> {
        use std::{io::Write, os::unix::fs::OpenOptionsExt};
        let _guard = LOCK.lock().map_err(|_| "Account storage unavailable")?;
        let path = path(key)?;
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|_| "Cannot create account storage")?;
        let tmp = path.with_extension("tmp");
        let result = (|| -> std::io::Result<()> {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&tmp)?;
            f.write_all(secret.as_bytes())?;
            f.sync_all()?;
            std::fs::rename(&tmp, &path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(tmp);
        }
        result.map_err(|_| "Cannot save account".into())
    }
    fn delete(&self, key: &str) -> Result<(), String> {
        let _guard = LOCK.lock().map_err(|_| "Account storage unavailable")?;
        match std::fs::remove_file(path(key)?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("Cannot remove saved account".into()),
        }
    }
    fn persistent(&self) -> bool {
        true
    }
}
