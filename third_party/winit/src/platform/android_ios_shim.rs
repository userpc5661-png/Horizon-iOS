//! iOS stand-ins for the Android helper API Horizon's mobile client calls.
//!
//! Horizon shares one touch client between Android and iOS. Android routes
//! surface scaling, insets and device IDs through NativeActivity; UIKit gives
//! the full view to the renderer, so these keep the identity behaviour.

use std::sync::Mutex;

use crate::window::Window;

static INPUT_SCALE: Mutex<[f64; 2]> = Mutex::new([1.0, 1.0]);
static INSETS: Mutex<[u32; 4]> = Mutex::new([0, 0, 0, 0]);
static ASPECT: Mutex<[u32; 2]> = Mutex::new([0, 0]);

/// Android framework text routing; UIKit text input goes through `set_ime_allowed`.
pub fn set_native_text_input_active(_active: bool) {}

pub fn set_native_input_scale(scale: [f64; 2]) {
    *INPUT_SCALE.lock().unwrap() = scale;
}

pub fn native_input_scale() -> [f64; 2] {
    *INPUT_SCALE.lock().unwrap()
}

/// Physical safe insets in left, top, right, bottom order.
pub fn set_native_content_insets(insets: [u32; 4]) {
    *INSETS.lock().unwrap() = insets;
}

pub fn set_native_content_aspect(aspect: [u32; 2]) {
    *ASPECT.lock().unwrap() = aspect;
}

/// The presentation rectangle: x, y, width, height in surface pixels.
pub fn native_content_rect(size: [u32; 2]) -> [u32; 4] {
    let [w, h] = size;
    let [left, top, right, bottom] = *INSETS.lock().unwrap();
    let x = left.min(w);
    let y = top.min(h);
    let mut width = w.saturating_sub(left + right).max(1);
    let mut height = h.saturating_sub(top + bottom).max(1);
    let [aw, ah] = *ASPECT.lock().unwrap();
    if aw > 0 && ah > 0 {
        let fit_w = (u64::from(height) * u64::from(aw) / u64::from(ah)) as u32;
        if fit_w <= width {
            width = fit_w.max(1);
        } else {
            height = ((u64::from(width) * u64::from(ah) / u64::from(aw)) as u32).max(1);
        }
    }
    [x, y, width, height]
}

pub fn native_content_origin() -> [u32; 2] {
    let insets = *INSETS.lock().unwrap();
    [insets[0], insets[1]]
}

pub fn native_device_id(_id: crate::event::DeviceId) -> i32 {
    0
}

pub fn set_native_gamepads(_ids: &[i32]) {}

pub fn native_pointer_device(_id: crate::event::DeviceId) -> bool {
    false
}

/// Matches android-activity's `Rect`; zero means "use the whole view".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub trait WindowExtAndroid {
    fn set_render_size(
        &self,
        size: crate::dpi::PhysicalSize<u32>,
    ) -> std::io::Result<crate::dpi::PhysicalSize<u32>>;

    fn content_rect(&self) -> Rect;
}

impl WindowExtAndroid for Window {
    /// UIKit keeps the drawable at the view's size and reports touches in it,
    /// so a smaller logical surface would misplace input: keep the view's size.
    fn set_render_size(
        &self,
        _size: crate::dpi::PhysicalSize<u32>,
    ) -> std::io::Result<crate::dpi::PhysicalSize<u32>> {
        Ok(self.inner_size())
    }

    fn content_rect(&self) -> Rect {
        Rect::default()
    }
}
