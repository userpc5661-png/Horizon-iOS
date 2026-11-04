//! Android reads ApplicationExitInfo; iOS reports crashes through its own logs.
use std::path::Path;

pub fn collect(_home: &Path) {}

pub fn install(home: &Path) {
    collect(home);
}
