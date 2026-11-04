fn main() {
    // iOS gives the main thread 1 MiB of stack, Android and desktop 8 MiB. The
    // game builds its large state on the main thread (winit runs it there), so
    // ask for 64 MiB through LC_MAIN, as macOS and iOS executables allow.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("ios") {
        println!("cargo:rustc-link-arg-bins=-Wl,-stack_size,0x4000000");
    }
}
