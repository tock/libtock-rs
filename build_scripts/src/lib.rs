//! Utility functions for implementing build.rs files for libtock-rs apps.

/// This path is set by this crate's build.rs file when this crate is compiled.
/// This allows this file to know the path of its own local outdir where copies
/// of linker scripts are stored (by this crate's build.rs). That path can then
/// be used in other libtock-rs compilations to provide useful linker scripts.
pub const BUILD_SCRIPTS_OUT_DIR: &str = env!("LIBTOCK_BUILD_SCRIPTS_OUT_DIR");

/// Helper function to configure cargo to use suitable linker scripts for
/// linking libtock-rs apps.
///
/// This function does two things:
///
/// 1. Make sure that the linker's search path includes where
///    `libtock_layout.ld` is stored. This is a general linker script designed
///    for libtock-rs apps and the Tock kernel.
///
/// 2. Reference a board-specific linker script that essentially sets the
///    `MEMORY` command to specify the flash and RAM addresses where the app
///    should be compiled. This happens by passing the `-T<linker script.ld>`
///    flag to the linker.
///
///    This function supports two methods for doing this:
///
///    1. Passing the `LIBTOCK_LIBTOCK_PLATFORM` environment variable which
///       specifies the name of the linker script in `/layouts` to be used.
///
///    2. Passing the `LIBTOCK_LINKER_FLASH` and `LINKER_RAM` environment
///       variables which specify the starting addresses of flash and RAM
///       memory, respectively.
pub fn auto_layout() {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    const PLATFORM_CFG_VAR: &str = "LIBTOCK_PLATFORM";
    const LINKER_FLASH_CFG_VAR: &str = "LIBTOCK_LINKER_FLASH";
    const LINKER_RAM_CFG_VAR: &str = "LIBTOCK_LINKER_RAM";

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

    // Set the linker search path to the out dir of this crate where we have
    // stored all of the linker files.
    println!("cargo:rustc-link-search={}", BUILD_SCRIPTS_OUT_DIR);

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
        // Point the linker to the correct platform-specific linker file.
        let platform_ld_name = format!("{}.ld", platform);
        println!("cargo:rustc-link-arg=-T{}", platform_ld_name);
    } else if let (Ok(linker_flash), Ok(linker_ram)) = (linker_flash, linker_ram) {
        // Create a valid linker file with the specified flash and ram locations.
        //
        // ```
        // TBF_HEADER_SIZE = 0x80;
        //
        // FLASH_START = 0x00040000;
        // FLASH_LENGTH = 0x00040000;
        //
        // RAM_START = 0x20008000;
        // RAM_LENGTH = 62K;
        //
        // INCLUDE libtock_layout.ld
        // ```
        let linker_script_name = format!("{}.{}.ld", linker_flash, linker_ram);
        let out_platform_path: PathBuf = [out_dir, &linker_script_name].iter().collect();
        let mut file = File::create(out_platform_path).expect("Could not create linker file");
        writeln!(file, "TBF_HEADER_SIZE = 0x80;").expect("Could not write linker file");
        writeln!(file, "FLASH_START = {};", linker_flash).expect("Could not write linker file");
        writeln!(file, "FLASH_LENGTH = 0x000D0000;",).expect("Could not write linker file");
        writeln!(file, "RAM_START = {};", linker_ram).expect("Could not write linker file");
        writeln!(file, "RAM_LENGTH = 46K;",).expect("Could not write linker file");
        writeln!(file, "INCLUDE libtock_layout.ld").expect("Could not write linker file");

        // Pass the name of this linker script to rustc.
        println!("cargo:rustc-link-arg=-T{}", linker_script_name);

        // Tell rustc where to search for the layout file.
        println!("cargo:rustc-link-search={}", out_dir);
    } else {
        panic!("Need to set LIBTOCK_PLATFORM or (LIBTOCK_LINKER_FLASH and LIBTOCK_LINKER_RAM)");
    }
}
