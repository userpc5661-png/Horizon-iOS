//! iOS system services with the Android platform module's API.
use horizon_client::osk::Output;
use std::cell::RefCell;
thread_local! {
    static EDIT: RefCell<Option<String>> = const { RefCell::new(None) };
    static MULTILINE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
static CACHE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

pub fn initialize(cache: std::path::PathBuf) {
    let _ = CACHE.set(cache);
    crate::objc::set_idle_timer_disabled(true);
}

pub fn vibrate(pulse: horizon_client::gamepad::Pulse) {
    if pulse.millis == 0 || pulse.strong.max(pulse.weak) <= 0.0 {
        return;
    }
    crate::objc::impact();
}
pub fn cancel_vibration() {}

pub fn clipboard_get() -> Option<String> {
    crate::objc::pasteboard_text()
}
pub fn clipboard_set(text: &str) {
    let _ = crate::objc::set_pasteboard_text(text);
}
pub fn copy_login_code(text: &str) -> bool {
    crate::objc::set_pasteboard_text(text)
}

/// Orientation comes from Info.plist (landscape); UIKit has no per-call request
/// without a scene-based app.
pub fn set_orientation(_orientation: crate::catalog::Orientation) {}

pub fn cache_dir() -> Option<std::path::PathBuf> {
    CACHE.get().cloned()
}

pub fn open_repository() -> bool {
    open_url("https://github.com/Vasysik/horizon-android")
}
/// The player's files are in the Files app, under On My iPhone > Horizon.
pub fn open_documents(_id: &str) -> bool {
    open_url("shareddocuments://")
}
pub fn open_url(url: &str) -> bool {
    if !url.starts_with("https://") && !url.starts_with("http://") && url != "shareddocuments://"
    {
        return false;
    }
    crate::objc::open_url(url)
}
pub fn performance_power() -> serde_json::Value {
    serde_json::json!({"platform": "ios"})
}

pub fn active() -> bool {
    EDIT.with(|e| e.borrow().is_some())
}

/// iOS apps do not quit themselves; the launcher stays on screen.
pub fn finish() {}

pub fn capture_mouse(capture: bool) -> bool {
    crate::native_ime::capture_mouse(capture)
}

pub fn show(text: &str) {
    show_editor(text, false);
}
pub fn show_secret(text: &str) {
    MULTILINE.with(|mode| mode.set(false));
    if crate::native_ime::show_secret(text) {
        EDIT.with(|e| *e.borrow_mut() = Some(text.into()));
    }
}
pub fn show_multiline(text: &str) {
    show_editor(text, true);
}
fn show_editor(text: &str, multiline: bool) {
    MULTILINE.with(|mode| mode.set(multiline));
    if crate::native_ime::show(text) {
        EDIT.with(|e| *e.borrow_mut() = Some(text.into()));
    } else {
        log::warn!("iOS keyboard is unavailable");
    }
}
pub fn hide() {
    EDIT.with(|e| *e.borrow_mut() = None);
    crate::native_ime::hide();
}

pub fn poll() -> Vec<Output> {
    if crate::native_ime::take_closed() && active() {
        EDIT.with(|e| *e.borrow_mut() = None);
        return vec![Output::Close];
    }
    EDIT.with(|edit| {
        let mut edit = edit.borrow_mut();
        let Some(old) = edit.as_mut() else {
            return vec![];
        };
        let Some(new) = crate::native_ime::text() else {
            return vec![];
        };
        if new == *old {
            return vec![];
        }
        let out =
            horizon_client::ime_edit::editor_changes(old, &new, MULTILINE.with(|mode| mode.get()));
        *old = new;
        out
    })
}

/// Field hit tests use the same GUI coordinates as the touch menus.
pub fn field_at(p: [f32; 2]) -> bool {
    horizon_client::widgets::last_targets().iter().any(|t| {
        let [x, y, w, h] = t.rect;
        t.kind == horizon_client::widgets::Kind::Field
            && (x..x + w).contains(&p[0])
            && (y..y + h).contains(&p[1])
    })
}

/// Only Microsoft's sign-in pages can be opened by the login panel.
pub fn open_login_url(url: &str) -> Result<(), String> {
    let fixed = [
        "https://www.microsoft.com/link",
        "https://microsoft.com/link",
        "https://microsoft.com/devicelogin",
        "https://www.microsoft.com/devicelogin",
        "https://aka.ms/mce-reviewappid",
    ]
    .contains(&url);
    let parsed = url::Url::parse(url).ok();
    let allowed = fixed
        || parsed.as_ref().is_some_and(|u| {
            u.scheme() == "https"
                && u.username().is_empty()
                && u.password().is_none()
                && u.port_or_known_default() == Some(443)
                && u.fragment().is_none()
                && match u.host_str() {
                    Some("microsoft.com" | "www.microsoft.com") => {
                        matches!(u.path(), "/link" | "/devicelogin")
                    }
                    Some("login.microsoftonline.com") => {
                        u.path() == "/consumers/oauth2/v2.0/authorize"
                    }
                    _ => false,
                }
        });
    if !allowed {
        return Err("Unexpected Microsoft sign-in URL.".into());
    }
    if crate::objc::open_url(url) {
        Ok(())
    } else {
        Err("Could not open Safari. Open microsoft.com/link and enter the copied code.".into())
    }
}

/// Controllers on iOS arrive through GameController.framework, not yet wired.
pub fn poll_gamepad_connections() {}

/// Info.plist allows landscape only; gyro axes follow landscape-right.
pub fn display_rotation() -> i32 {
    1
}
