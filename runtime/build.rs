// auto_layout() identifies the correct linker scripts to use based on the
// LIBTOCK_PLATFORM environment variable, and copies the linker scripts into
// OUT_DIR. The cargo invocation must pass -C link-arg=-Tlayout.ld to rustc
// (using the rustflags cargo config).
#[cfg(not(feature = "no_auto_layout"))]
fn auto_layout() {
    use std::fs::copy;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    const LINKER_FLASH_CFG_VAR: &str = "LINKER_FLASH";
    const LINKER_RAM_CFG_VAR: &str = "LINKER_RAM";
    const LAYOUT_GENERIC_FILENAME: &str = "libtock_layout.ld";

    // Note: we need to print these rerun-if commands before using the variable
    // or file, so that if the build script fails cargo knows when to re-run it.
    // println!("cargo:rerun-if-env-changed={}", PLATFORM_CFG_VAR);

    // Read configuration from environment variables.

    // Note: cargo fails if run in a path that is not valid Unicode, so this
    // script doesn't need to handle non-Unicode paths. Also, OUT_DIR cannot be
    // in a location with a newline in it, or we have no way to pass
    // rustc-link-search to cargo.
    let out_dir = &std::env::var("OUT_DIR").expect("Unable to read OUT_DIR");
    assert!(
        !out_dir.contains('\n'),
        "Build path contains a newline, which is unsupported"
    );

    // Read the platform environment variable as a String (our platform names
    // should all be valid UTF-8).
    let linker_flash = std::env::var(LINKER_FLASH_CFG_VAR).expect("Please specify LINKER_FLASH");
    let linker_ram = std::env::var(LINKER_RAM_CFG_VAR).expect("Please specify LINKER_RAM");

    // Create a valid linker file with the specified flash and ram locations.
    //
    // ```
    // MEMORY {
    //   FLASH (X) : ORIGIN = $LINKER_FLASH, LENGTH = 0x000D0000
    //   RAM   (W) : ORIGIN = $LINKER_RAM,   LENGTH = 46K
    // }
    // TBF_HEADER_SIZE = 0x60;
    // INCLUDE libtock_layout.ld
    // ```
    let out_platform_path: PathBuf = [out_dir, "layout.ld"].iter().collect();
    let mut file = File::create(out_platform_path).expect("Could not create linker file");
    write!(file, "MEMORY {{\n").expect("Could not write linker file");
    write!(
        file,
        "  FLASH (X) : ORIGIN = {}, LENGTH = 0x000D0000\n",
        linker_flash
    )
    .expect("Could not write linker file");
    write!(
        file,
        "  RAM   (X) : ORIGIN = {}, LENGTH = 46k\n",
        linker_ram
    )
    .expect("Could not write linker file");
    write!(file, "}}\n").expect("Could not write linker file");
    write!(file, "TBF_HEADER_SIZE = 0x60;\n").expect("Could not write linker file");
    write!(file, "INCLUDE libtock_layout.ld\n").expect("Could not write linker file");

    // Copy the generic layout file into OUT_DIR.
    let out_layout_generic: PathBuf = [out_dir, LAYOUT_GENERIC_FILENAME].iter().collect();
    println!("cargo:rerun-if-changed={}", LAYOUT_GENERIC_FILENAME);
    copy(LAYOUT_GENERIC_FILENAME, out_layout_generic)
        .expect("Unable to copy layout_generic.ld into OUT_DIR");

    // Tell rustc where to search for the layout file.
    println!("cargo:rustc-link-search={}", out_dir);
}

fn main() {
    #[cfg(not(feature = "no_auto_layout"))]
    auto_layout();
}
