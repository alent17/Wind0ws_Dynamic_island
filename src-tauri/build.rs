fn main() {
    tauri_build::build();

    // `tauri_build` links its resource library into desktop bins, but not the
    // library unit-test harness. Expose the generated library to the test-only
    // link attribute in lib.rs.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!(
            "cargo:rustc-link-search=native={}",
            std::env::var_os("OUT_DIR").unwrap().to_string_lossy()
        );
    }
}
