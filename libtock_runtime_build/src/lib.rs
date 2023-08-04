//! Utility functions for implementing build.rs files for libtock-rs apps.

pub fn auto_layout() {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    const PLATFORM_CFG_VAR: &str = "LIBTOCK_PLATFORM";
    const LINKER_FLASH_CFG_VAR: &str = "LINKER_FLASH";
    const LINKER_RAM_CFG_VAR: &str = "LINKER_RAM";

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

    // Point the linker to the generic libtock-rs linker file.
    let generic_ld_dir: PathBuf = ["runtime"].iter().collect();
    println!("cargo:rustc-link-search={}", generic_ld_dir.display());

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
        let platform_ld_dir: PathBuf = ["runtime", "layouts"].iter().collect();
        let platform_ld_path: PathBuf = ["runtime", "layouts", &platform_ld_name].iter().collect();

        assert!(platform_ld_path.exists(), "Unknown platform {}", platform);

        println!("cargo:rerun-if-changed={}", platform_ld_path.display());
        println!("cargo:rustc-link-arg=-T{}", platform_ld_name);

        // Tell rustc where to search for the layout file.
        println!("cargo:rustc-link-search={}", platform_ld_dir.display());
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
        write!(file, "TBF_HEADER_SIZE = 0x80;\n").expect("Could not write linker file");
        write!(file, "FLASH_START = {};\n", linker_flash).expect("Could not write linker file");
        write!(file, "FLASH_LENGTH = 0x000D0000;\n",).expect("Could not write linker file");
        write!(file, "RAM_START = {};\n", linker_ram).expect("Could not write linker file");
        write!(file, "RAM_LENGTH = 46K;\n",).expect("Could not write linker file");
        write!(file, "INCLUDE libtock_layout.ld\n").expect("Could not write linker file");

        // Pass the name of this linker script to rustc.
        println!("cargo:rustc-link-arg=-T{}", linker_script_name);

        // Tell rustc where to search for the layout file.
        println!("cargo:rustc-link-search={}", out_dir);
    } else {
        panic!("Need to set LIBTOCK_PLATFORM or (LINKER_FLASH and LINKER_RAM)");
    }
}
