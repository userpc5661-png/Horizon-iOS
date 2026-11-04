// The iOS app, included by ios_main.rs on iOS only. The launcher, touch client,
// simulation and renderer are the Android port's; only the platform services
// under `ios/` differ.

#[allow(dead_code, clippy::collapsible_match)]
// Shared vanilla match structure and desktop harnesses.
mod game;
#[path = "android/game_launch.rs"]
mod game_launch;
#[path = "android/launcher.rs"]
mod launcher;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/account.rs"]
mod account;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/browser.rs"]
mod browser;
#[path = "android/catalog.rs"]
mod catalog;
#[path = "android/device_preferences.rs"]
mod device_preferences;
#[path = "ios/device_runtime.rs"]
mod device_runtime;
#[path = "android/diagnostics.rs"]
mod diagnostics;
#[path = "android/display.rs"]
mod display;
#[path = "ios/document_provider.rs"]
mod document_provider;
#[path = "android/document_store.rs"]
mod document_store;
#[path = "android/documents.rs"]
mod documents;
#[path = "ios/exit_reports.rs"]
mod exit_reports;
#[path = "ios/gallery.rs"]
mod gallery;
#[path = "ios/native_ime.rs"]
mod native_ime;
#[path = "ios/objc.rs"]
mod objc;
#[path = "android/performance.rs"]
mod performance;
#[path = "ios/platform.rs"]
mod platform;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/proxy.rs"]
mod proxy;
#[path = "android/search_queue.rs"]
mod search_queue;
#[path = "ios/secure_store.rs"]
mod secure_store;
#[path = "android/shader_catalog.rs"]
mod shader_catalog;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../launcher/src/skins.rs"]
mod skins;
// The shared account protocol needs only the launcher's version and data path.
mod backend {
    pub const VERSION: &str = if crate::PUBLIC_BUILD {
        "0.7.7"
    } else {
        "0.7.7-dev"
    };
    static HOME: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    pub fn initialize(home: std::path::PathBuf) {
        let _ = HOME.set(home);
    }
    pub fn game_home() -> std::path::PathBuf {
        data_dir()
    }
    pub fn game_mod_api_level() -> Option<u32> {
        Some(horizon_mods::api_level::LEVEL)
    }
    pub fn data_dir() -> std::path::PathBuf {
        HOME.get().expect("ios_main sets account home").clone()
    }
}

pub(crate) fn keyboard_mouse_input() -> bool {
    horizon_client::android_runtime::active_input() == catalog::InputHost::KeyboardMouse
}
pub(crate) fn configure_input_mode(mode: catalog::InputHost) {
    horizon_client::android_runtime::configure_input(mode);
}
pub(crate) fn touch_visible() -> bool {
    horizon_client::android_runtime::active_input() == catalog::InputHost::Touch
}
pub(crate) fn touch_input(touch: bool) {
    horizon_client::android_runtime::note_input(if touch {
        catalog::InputHost::Touch
    } else {
        catalog::InputHost::KeyboardMouse
    });
}
pub(crate) fn save_input_mode() {
    let home = backend::data_dir();
    let mut config = catalog::Config::load(&home);
    let mode = horizon_client::android_runtime::input_mode();
    if config.input_host != mode {
        config.input_host = mode;
        if let Err(e) = config.save(&home) {
            log::warn!("Saving input mode: {e}");
        }
    }
}
pub(crate) const PUBLIC_BUILD: bool = match option_env!("HORIZON_ANDROID_PUBLIC") {
    Some(value) => value.len() == 1 && value.as_bytes()[0] == b'1',
    None => false,
};

use winit::event_loop::EventLoop;

thread_local! {
    static WINDOW: std::cell::RefCell<Option<std::sync::Arc<winit::window::Window>>> =
        const { std::cell::RefCell::new(None) };
}
/// One UIWindow for the app's life. The launcher and the game each drop their
/// handle when they hand over (Android's NativeActivity has a single window
/// too); a second UIWindow left behind a detached view that crashed UIKit.
pub(crate) fn ios_window(
    events: &winit::event_loop::ActiveEventLoop,
    attributes: winit::window::WindowAttributes,
) -> Result<std::sync::Arc<winit::window::Window>, winit::error::OsError> {
    WINDOW.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(window) = slot.as_ref() {
            return Ok(window.clone());
        }
        let window = std::sync::Arc::new(events.create_window(attributes)?);
        *slot = Some(window.clone());
        Ok(window)
    })
}

pub(crate) fn android_event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    EventLoop::new()
}

/// stderr reaches Xcode and Console.app; diagnostics keeps the in-app log.
fn init_logging() {
    struct LocalLogger;
    impl log::Log for LocalLogger {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::Level::Info
                || metadata.target().starts_with("horizon")
                    && metadata.level() <= log::Level::Debug
        }
        fn log(&self, record: &log::Record<'_>) {
            if !self.enabled(record.metadata()) {
                return;
            }
            let text = record.args().to_string();
            eprintln!("[Horizon {} {}] {text}", record.level(), record.target());
            diagnostics::log_line(record.level(), record.target(), &text);
        }
        fn flush(&self) {
            diagnostics::request_flush();
        }
    }
    static LOGGER: LocalLogger = LocalLogger;
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}

/// The app sandbox: Documents is the player's (visible in the Files app),
/// Library/Caches may be purged by iOS, Library/Application Support is private.
fn sandbox() -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    let root = std::path::PathBuf::from(std::env::var_os("HOME")?);
    Some((root.join("Documents"), root.join("Library/Caches/Horizon")))
}

fn main() {
    init_logging();
    let Some((home, cache)) = sandbox() else {
        eprintln!("Horizon: iOS supplied no application sandbox");
        return;
    };
    for dir in [&home, &cache] {
        if let Err(error) = std::fs::create_dir_all(dir) {
            eprintln!("Horizon: cannot create {}: {error}", dir.display());
            return;
        }
    }
    backend::initialize(home.clone());
    if let Err(error) = diagnostics::initialize(&home, serde_json::json!({"platform": "ios"})) {
        log::error!("Cannot initialize local diagnostics: {error}");
    }
    platform::initialize(cache.clone());
    horizon_client::android_runtime::configure_cache_dir(cache.clone());
    horizon_client::official_install::configure_download_cache_dir(cache);
    horizon_client::android_runtime::install_clipboard(
        horizon_client::android_runtime::ClipboardHooks {
            get: platform::clipboard_get,
            set: platform::clipboard_set,
        },
    );
    if let Err(error) = std::env::set_current_dir(&home) {
        log::error!("Cannot open {}: {error}", home.display());
        return;
    }
    horizon_client::crash::install(home.clone(), "client");
    exit_reports::install(&home);
    let _ = std::fs::remove_file(home.join("android-error.txt"));
    // The bundle is read-only; the launcher and installer expect the engine
    // data beside the player's files, as the APK's asset is copied on Android.
    if let Some(bundled) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("engine-data.zip")))
        .filter(|path| path.is_file())
    {
        let target = home.join("engine-data.zip");
        let stale = match (std::fs::metadata(&bundled), std::fs::metadata(&target)) {
            (Ok(a), Ok(b)) => a.len() != b.len() || a.modified().ok() > b.modified().ok(),
            _ => true,
        };
        if stale {
            let partial = home.join("engine-data.partial");
            let copied = std::fs::copy(&bundled, &partial)
                .and_then(|_| std::fs::rename(&partial, &target));
            if let Err(error) = copied {
                log::error!("Cannot unpack engine data: {error}");
            }
        }
    } else {
        log::error!("The app bundle has no engine-data.zip");
    }
    let mut launcher = launcher::Launcher::new(home.clone());
    // On iOS run_app hands the thread to UIApplicationMain and does not return.
    let result = android_event_loop()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })
        .and_then(|events| events.run_app(&mut launcher).map_err(Into::into));
    if let Err(error) = result {
        log::error!("Horizon: {error}");
        diagnostics::record_error(&error.to_string());
        horizon_client::crash::report_error(&home, "client", &*error);
        let _ = std::fs::write(home.join("android-error.txt"), error.to_string());
    }
    diagnostics::finish();
}
