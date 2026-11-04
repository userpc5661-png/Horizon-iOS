//! Screenshots stay in the game folder on iOS, which the Files app shows.
use std::path::Path;

pub fn save(file: &Path) -> std::result::Result<(), String> {
    if file.is_file() {
        Ok(())
    } else {
        Err(format!("{} does not exist", file.display()))
    }
}

pub fn poll() -> Vec<std::result::Result<String, String>> {
    Vec::new()
}
