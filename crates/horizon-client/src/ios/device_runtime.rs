//! Android's frame-rate requests, ADPF hints and gyro. UIKit paces frames
//! through CAMetalLayer itself, so iOS keeps only the shared preferences.
use std::time::Instant;

pub struct Frame {
    _start: Instant,
}
/// The actual duration is CPU-side frame work including submission waits.
pub fn frame(_enabled: bool, _hz: u16, _active: bool) -> Frame {
    Frame {
        _start: Instant::now(),
    }
}
impl Drop for Frame {
    fn drop(&mut self) {}
}
/// Gyro aiming needs CoreMotion, not wired on iOS yet.
pub fn gyro(_enabled: bool, _sensitivity: f32, _rotation: i32) -> [f32; 2] {
    [0.0; 2]
}
pub fn diagnostics() -> serde_json::Value {
    serde_json::json!({
        "platform": "ios",
        "gyro_available": false,
    })
}
pub fn suspend() {}

static PREFERENCES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub fn configure(mut settings: crate::device_preferences::Settings) {
    settings.sanitize();
    let bits = u32::from(settings.display_hz)
        | (u32::from(settings.performance_hints) << 16)
        | (u32::from(settings.gpu_profile) << 17);
    PREFERENCES.store(bits, std::sync::atomic::Ordering::Relaxed);
}
pub fn preferences() -> crate::device_preferences::Settings {
    let bits = PREFERENCES.load(std::sync::atomic::Ordering::Relaxed);
    crate::device_preferences::Settings {
        display_hz: bits as u16,
        performance_hints: bits & (1 << 16) != 0,
        gpu_profile: bits & (1 << 17) != 0,
    }
}
