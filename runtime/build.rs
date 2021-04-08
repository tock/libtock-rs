use std::fs::copy;
use std::path::PathBuf;

mod extern_asm;

// auto_layout() identifies the correct linker scripts to use based on the
// LIBTOCK_PLATFORM environment variable, and copies the linker scripts into
// OUT_DIR. The cargo invocation must pass -C link-arg=-Tlayout.ld to rustc
// (using the rustflags cargo config).
#[cfg(not(feature = "no_auto_layout"))]
fn auto_layout(out_dir: &str) {
    const PLATFORM_CFG_VAR: &str = "LIBTOCK_PLATFORM";
    const LAYOUT_GENERIC_FILENAME: &str = "libtock_layout.ld";

    // Note: we need to print these rerun-if commands before using the variable
    // or file, so that if the build script fails cargo knows when to re-run it.
    println!("cargo:rerun-if-env-changed={}", PLATFORM_CFG_VAR);

    // Read configuration from environment variables.

    // Read the platform environment variable as a String (our platform names
    // should all be valid UTF-8).
    let platform = std::env::var(PLATFORM_CFG_VAR).expect("Please specify LIBTOCK_PLATFORM");

    // Copy the platform-specific layout file into OUT_DIR.
    let platform_filename = format!("{}.ld", platform);
    let platform_path: PathBuf = ["layouts", &platform_filename].iter().collect();
    println!("cargo:rerun-if-changed={}", platform_path.display());
    assert!(platform_path.exists(), "Unknown platform {}", platform);
    let out_platform_path: PathBuf = [out_dir, "layout.ld"].iter().collect();
    copy(&platform_path, out_platform_path).expect("Unable to copy platform layout into OUT_DIR");

    // Copy the generic layout file into OUT_DIR.
    let out_layout_generic: PathBuf = [out_dir, LAYOUT_GENERIC_FILENAME].iter().collect();
    println!("cargo:rerun-if-changed={}", LAYOUT_GENERIC_FILENAME);
    copy(LAYOUT_GENERIC_FILENAME, out_layout_generic)
        .expect("Unable to copy layout_generic.ld into OUT_DIR");
}

fn main() {
    // Note: cargo fails if run in a path that is not valid Unicode, so this
    // script doesn't need to handle non-Unicode paths. Also, OUT_DIR cannot be
    // in a location with a newline in it, or we have no way to pass
    // rustc-link-search to cargo.
    let out_dir = &std::env::var("OUT_DIR").expect("Unable to read OUT_DIR");
    assert!(
        !out_dir.contains('\n'),
        "Build path contains a newline, which is unsupported"
    );

    #[cfg(not(feature = "no_auto_layout"))]
    auto_layout(out_dir);

    extern_asm::build_and_link(out_dir);

    // This link search path is used by both auto_layout() and
    // extern_asm::build_and_link().
    println!("cargo:rustc-link-search={}", out_dir);
}
