//! Android exposes the game folder through a DocumentsProvider. On iOS the
//! Files app reads Documents directly (UIFileSharingEnabled), so there are no
//! provider writes to report.
pub fn take_changes() -> bool {
    false
}
