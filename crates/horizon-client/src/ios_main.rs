//! iOS's native entry point. Other targets build an empty binary, so
//! workspace-wide builds and tests on desktop are unaffected.

#[cfg(target_os = "ios")]
include!("ios_app.rs");

#[cfg(not(target_os = "ios"))]
fn main() {}
