//! The UIKit keyboard as the Android port's native editor: a text buffer that
//! winit's `insertText:`/`deleteBackward` events edit, diffed by `platform::poll`.
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static TEXT: Mutex<Option<String>> = Mutex::new(None);
static CLOSED: AtomicBool = AtomicBool::new(false);

fn buffer() -> std::sync::MutexGuard<'static, Option<String>> {
    TEXT.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn show(text: &str) -> bool {
    *buffer() = Some(text.into());
    CLOSED.store(false, Ordering::Release);
    let shown = crate::objc::set_keyboard(true);
    if !shown {
        *buffer() = None;
    }
    shown
}
pub fn show_secret(text: &str) -> bool {
    show(text)
}
pub fn hide() {
    let open = buffer().take().is_some();
    if open {
        crate::objc::set_keyboard(false);
    }
}
pub fn text() -> Option<String> {
    buffer().clone()
}
/// Whether the player closed the keyboard since the last call.
pub fn take_closed() -> bool {
    CLOSED.swap(false, Ordering::AcqRel)
}

/// Keyboard events while a field is open belong to the editor, not the game.
pub fn window_event(event: &winit::event::WindowEvent) -> bool {
    use winit::{
        event::{ElementState, WindowEvent},
        keyboard::{Key, NamedKey},
    };
    let mut text = buffer();
    let Some(edit) = text.as_mut() else {
        return false;
    };
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            if event.state == ElementState::Pressed {
                match &event.logical_key {
                    Key::Named(NamedKey::Backspace) => {
                        edit.pop();
                    }
                    Key::Named(NamedKey::Enter) => edit.push('\n'),
                    _ => {
                        if let Some(typed) = &event.text {
                            edit.push_str(&typed.replace('\r', "\n"));
                        }
                    }
                }
            }
            true
        }
        WindowEvent::Ime(winit::event::Ime::Commit(typed)) => {
            edit.push_str(typed);
            true
        }
        WindowEvent::Ime(winit::event::Ime::Disabled) => {
            text.take();
            CLOSED.store(true, Ordering::Release);
            true
        }
        _ => false,
    }
}

/// iOS has no pointer capture for touch; hardware mice are not supported yet.
pub fn capture_mouse(_capture: bool) -> bool {
    false
}

/// The keyboard overlays the game; the launcher keeps its full height.
pub fn viewport_bottom() -> Option<u32> {
    None
}

/// The system document picker is not wired yet; imports use the Files app.
pub fn pick_document() -> bool {
    false
}

pub fn configure_screen(_immersive: bool, _avoid_cutout: bool) {}

/// UIKit presents the drawable over the whole view, notch included; HUD
/// margins in the game's options keep controls clear of it.
pub fn refresh_bounds() {}
