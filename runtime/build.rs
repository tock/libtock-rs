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

    const PLATFORM_CFG_VAR: &str = "LIBTOCK_PLATFORM";
    const LINKER_FLASH_CFG_VAR: &str = "LINKER_FLASH";
    const LINKER_RAM_CFG_VAR: &str = "LINKER_RAM";
    const LAYOUT_GENERIC_FILENAME: &str = "libtock_layout.ld";

    // Note: we need to print these rerun-if commands before using the variable
    // or file, so that if the build script fails cargo knows when to re-run it.
    println!("cargo:rerun-if-env-changed={}", PLATFORM_CFG_VAR);
    println!("cargo:rerun-if-env-changed={}", LINKER_FLASH_CFG_VAR);
    println!("cargo:rerun-if-env-changed={}", LINKER_RAM_CFG_VAR);

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

    // Where we are going to put the custom linker script for this particular
    // build.
    let out_platform_path: PathBuf = [out_dir, "layout.ld"].iter().collect();

    // Choose the linker file we are going to use for this build. That can be
    // specified by choosing a platform, where the linker file will be selected
    // from `runtime/layouts`, or by explicitly setting the flash and RAM
    // addresses.

    // Read the platform environment variable as a String (our platform names
    // should all be valid UTF-8).
    let platform = std::env::var(PLATFORM_CFG_VAR);

    // Read the explicit flash and RAM addresses.
    let linker_flash = std::env::var(LINKER_FLASH_CFG_VAR);
    let linker_ram = std::env::var(LINKER_RAM_CFG_VAR);

    if let Ok(platform) = platform {
        // Copy the platform-specific layout file into OUT_DIR.
        let platform_filename = format!("{}.ld", platform);
        let platform_path: PathBuf = ["layouts", &platform_filename].iter().collect();
        println!("cargo:rerun-if-changed={}", platform_path.display());
        assert!(platform_path.exists(), "Unknown platform {}", platform);
        copy(&platform_path, out_platform_path)
            .expect("Unable to copy platform layout into OUT_DIR");
    } else if let (Ok(linker_flash), Ok(linker_ram)) = (linker_flash, linker_ram) {
        // Create a valid linker file with the specified flash and ram locations.
        //
        // ```
        // TBF_HEADER_SIZE = 0x60;
        //
        // FLASH_START = 0x00040000;
        // FLASH_LENGTH = 0x00040000;
        //
        // RAM_START = 0x20008000;
        // RAM_LENGTH = 62K;
        //
        // INCLUDE libtock_layout.ld
        // ```
        let out_platform_path: PathBuf = [out_dir, "layout.ld"].iter().collect();
        let mut file = File::create(out_platform_path).expect("Could not create linker file");
        write!(file, "TBF_HEADER_SIZE = 0x60;\n").expect("Could not write linker file");
        write!(file, "FLASH_START = {};\n", linker_flash).expect("Could not write linker file");
        write!(file, "FLASH_LENGTH = 0x000D0000;\n",).expect("Could not write linker file");
        write!(file, "RAM_START = {};\n", linker_ram).expect("Could not write linker file");
        write!(file, "RAM_LENGTH = 46K;\n",).expect("Could not write linker file");
        write!(file, "INCLUDE libtock_layout.ld\n").expect("Could not write linker file");
    } else {
        panic!("Need to set LIBTOCK_PLATFORM or (LINKER_FLASH and LINKER_RAM)");
    }

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
