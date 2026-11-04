//! The few UIKit calls the iOS port makes, through the Objective-C runtime.
//! Every function here must run on the main thread, where winit calls the app.
use objc2::{
    msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject, Bool},
};
use objc2_foundation::NSString;

fn class(name: &std::ffi::CStr) -> Option<&'static AnyClass> {
    let class = AnyClass::get(name);
    if class.is_none() {
        log::warn!("iOS: class {name:?} is unavailable");
    }
    class
}

fn string(object: *mut AnyObject) -> Option<String> {
    if object.is_null() {
        return None;
    }
    // SAFETY: callers pass an NSString returned by UIKit, alive for this call.
    Some(unsafe { &*(object as *const NSString) }.to_string())
}

pub fn pasteboard_text() -> Option<String> {
    let class = class(c"UIPasteboard")?;
    unsafe {
        let board: *mut AnyObject = msg_send![class, generalPasteboard];
        if board.is_null() {
            return None;
        }
        let text: *mut AnyObject = msg_send![board, string];
        string(text)
    }
}

pub fn set_pasteboard_text(text: &str) -> bool {
    let Some(class) = class(c"UIPasteboard") else {
        return false;
    };
    let value = NSString::from_str(text);
    unsafe {
        let board: *mut AnyObject = msg_send![class, generalPasteboard];
        if board.is_null() {
            return false;
        }
        let _: () = msg_send![board, setString: &*value];
    }
    true
}

/// `UIApplication.open(_:options:completionHandler:)`.
pub fn open_url(url: &str) -> bool {
    let (Some(application), Some(urls), Some(dictionaries)) = (
        class(c"UIApplication"),
        class(c"NSURL"),
        class(c"NSDictionary"),
    ) else {
        return false;
    };
    let text = NSString::from_str(url);
    unsafe {
        let app: *mut AnyObject = msg_send![application, sharedApplication];
        let target: *mut AnyObject = msg_send![urls, URLWithString: &*text];
        let options: *mut AnyObject = msg_send![dictionaries, dictionary];
        if app.is_null() || target.is_null() || options.is_null() {
            return false;
        }
        let none: *mut AnyObject = std::ptr::null_mut();
        let _: () = msg_send![app, openURL: target, options: options, completionHandler: none];
    }
    true
}

/// The view winit draws into: the key window's root view controller's view.
/// It adopts UIKeyInput, so first-responder status shows the system keyboard.
fn game_view() -> Option<*mut AnyObject> {
    let application = class(c"UIApplication")?;
    unsafe {
        let app: *mut AnyObject = msg_send![application, sharedApplication];
        if app.is_null() {
            return None;
        }
        let window: *mut AnyObject = msg_send![app, keyWindow];
        if window.is_null() {
            return None;
        }
        let controller: *mut AnyObject = msg_send![window, rootViewController];
        if controller.is_null() {
            return None;
        }
        let view: *mut AnyObject = msg_send![controller, view];
        (!view.is_null()).then_some(view)
    }
}

pub fn set_keyboard(visible: bool) -> bool {
    let Some(view) = game_view() else {
        return false;
    };
    unsafe {
        let done: Bool = if visible {
            msg_send![view, becomeFirstResponder]
        } else {
            msg_send![view, resignFirstResponder]
        };
        done.as_bool() || !visible
    }
}

/// A short tap through `UIImpactFeedbackGenerator` (its default, medium style).
pub fn impact() {
    let Some(class) = class(c"UIImpactFeedbackGenerator") else {
        return;
    };
    unsafe {
        let generator: Option<Retained<AnyObject>> = msg_send![class, new];
        if let Some(generator) = generator {
            let _: () = msg_send![&*generator, impactOccurred];
        }
    }
}

/// The view's safe-area insets in points: left, top, right, bottom.
pub fn safe_area_points() -> Option<[f64; 4]> {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Insets {
        top: f64,
        left: f64,
        bottom: f64,
        right: f64,
    }
    use objc2::encode::{Encode, Encoding};
    unsafe impl Encode for Insets {
        const ENCODING: Encoding = Encoding::Struct(
            "UIEdgeInsets",
            &[f64::ENCODING, f64::ENCODING, f64::ENCODING, f64::ENCODING],
        );
    }
    let view = game_view()?;
    let insets: Insets = unsafe { msg_send![view, safeAreaInsets] };
    Some([insets.left, insets.top, insets.right, insets.bottom])
}

/// UIScreen.main.nativeScale: physical pixels per point.
pub fn screen_scale() -> f64 {
    let Some(class) = class(c"UIScreen") else {
        return 1.0;
    };
    unsafe {
        let screen: *mut AnyObject = msg_send![class, mainScreen];
        if screen.is_null() {
            return 1.0;
        }
        let scale: f64 = msg_send![screen, nativeScale];
        if scale > 0.0 { scale } else { 1.0 }
    }
}

/// Keep the screen on while playing, as Android's NativeActivity window does.
pub fn set_idle_timer_disabled(disabled: bool) {
    let Some(application) = class(c"UIApplication") else {
        return;
    };
    unsafe {
        let app: *mut AnyObject = msg_send![application, sharedApplication];
        if !app.is_null() {
            let _: () = msg_send![app, setIdleTimerDisabled: Bool::new(disabled)];
        }
    }
}
