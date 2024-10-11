//! Utility functions for implementing build.rs files for libtock-rs apps.

/// List of known `LIBTOCK_PLATFORM` values.
#[rustfmt::skip]
const PLATFORMS: &[(&str, &str, &str, &str, &str)] = &[
    // Name               | Flash start | Flash len  | RAM start   | RAM length
    ("apollo3"            , "0x00040000", "0x00BE000", "0x10004000", "0x03000"),
    ("clue_nrf52840"      , "0x00080000", "512K"     , "0x20006000", "216K"   ),
    ("esp32_c3_devkitm_1" , "0x403B0000", "0x0030000", "0x3FCA2000", "0x2E000"),
    ("hail"               , "0x00030000", "0x0040000", "0x20008000", "62K"    ),
    ("hifive1"            , "0x20040000", "32M"      , "0x80003000", "0x01000"),
    ("imix"               , "0x00040000", "0x0040000", "0x20008000", "62K"    ),
    ("imxrt1050"          , "0x63002000", "0x1000000", "0x20004000", "112K"   ),
    ("microbit_v2"        , "0x00040000", "256K"     , "0x20004000", "112K"   ),
    ("msp432"             , "0x00020000", "0x0020000", "0x20004000", "0x02000"),
    ("nano_rp2040_connect", "0x10020000", "256K"     , "0x20004000", "248K"   ),
    ("nrf52"              , "0x00030000", "0x0060000", "0x20004000", "62K"    ),
    ("nrf52840"           , "0x00040000", "768K"     , "0x20010000", "128k"    ),
    ("nucleo_f429zi"      , "0x08040000", "255K"     , "0x20004000", "112K"   ),
    ("nucleo_f446re"      , "0x08040000", "255K"     , "0x20004000", "176K"   ),
    ("opentitan"          , "0x20030000", "32M"      , "0x10006000", "126K"   ),
    ("pico_explorer_base" , "0x10040000", "256K"     , "0x20012000", "192K"   ),
    ("qemu_rv32_virt"     , "0x80100000", "0x0100000", "0x8020a000", "0xf6000"),
    ("raspberry_pi_pico"  , "0x10040000", "256K"     , "0x20012000", "192K"   ),
    ("stm32f3discovery"   , "0x08020000", "0x0020000", "0x20004000", "48K"    ),
    ("stm32f412gdiscovery", "0x08030000", "256K"     , "0x20004000", "112K"   ),
    ("nano33ble"          , "0x00050000", "704K"     , "0x20005000", "240K"   ),
];

/// Helper function to configure cargo to use suitable linker scripts for
/// linking libtock-rs apps.
///
/// `auto_layout` function does a few things:
///
/// 1. Copies `libtock_layout.ld` into the linker's search path.
/// 2. Generates a linker script that specifies which flash and RAM addresses
///    the app should be compiled for. This linker script depends on the
///    previously-mentioned `libtock_layout.ld`.
/// 3. Passes the `-T<linker script.ld>` argument to the linker to make it use
///    the generated linker script.
///
/// `auto_layout` supports two mechanisms for specifying the flash and RAM
/// address ranges:
///
/// 1. Passing the `LIBTOCK_PLATFORM` environment variable, specifying one of a
///    hardcoded list of known platforms. See the `PLATFORMS` variable above for
///    the list of supported platforms.
/// 2. Passing the `LIBTOCK_LINKER_FLASH` and `LIBTOCK_LINKER_RAM` environment
///    variables which specify the starting addresses of flash and RAM memory,
///    respectively.
///
/// Programs passing `LIBTOCK_LINKER_FLASH` and `LIBTOCK_LINKER_RAM` may
/// additionally pass `LIBTOCK_TBF_HEADER_SIZE`, `LIBTOCK_LINKER_FLASH_LENGTH`,
/// and/or `LIBTOCK_LINKER_RAM_LENGTH`. If not specified, this function will
/// assume some default values for those variables.
pub fn auto_layout() {
    use std::env::var;
    use std::fs::File;
    use std::io::Write;
    use std::ops::Deref;
    use std::path::PathBuf;

    const LIBTOCK_LAYOUT_NAME: &str = "libtock_layout.ld";
    const LINKER_FLASH_VAR: &str = "LIBTOCK_LINKER_FLASH";
    const LINKER_FLASH_LEN_VAR: &str = "LIBTOCK_LINKER_FLASH_LENGTH";
    const LINKER_RAM_VAR: &str = "LIBTOCK_LINKER_RAM";
    const LINKER_RAM_LEN_VAR: &str = "LIBTOCK_LINKER_RAM_LENGTH";
    const PLATFORM_VAR: &str = "LIBTOCK_PLATFORM";
    const TBF_HEADER_SIZE_VAR: &str = "LIBTOCK_TBF_HEADER_SIZE";

    // Note: we need to print these rerun-if commands before using the variable
    // or file, so that if the build script fails cargo knows when to re-run it.
    println!("cargo:rerun-if-env-changed={}", LINKER_FLASH_VAR);
    println!("cargo:rerun-if-env-changed={}", LINKER_FLASH_LEN_VAR);
    println!("cargo:rerun-if-env-changed={}", LINKER_RAM_VAR);
    println!("cargo:rerun-if-env-changed={}", LINKER_RAM_LEN_VAR);
    println!("cargo:rerun-if-env-changed={}", PLATFORM_VAR);
    println!("cargo:rerun-if-env-changed={}", TBF_HEADER_SIZE_VAR);

    let platform = get_env_var(PLATFORM_VAR);
    let flash_start = get_env_var(LINKER_FLASH_VAR);
    let ram_start = get_env_var(LINKER_RAM_VAR);
    let flash_len;
    let ram_len;
    // Determine the flash and RAM address ranges. This detects whether
    // LIBTOCK_PLATFORM was specified or whether the flash and RAM ranges were
    // specified directly.
    let (flash_start, flash_len, ram_start, ram_len) = match (platform, &flash_start, &ram_start) {
        (None, Some(flash_start), Some(ram_start)) => {
            // The flash and RAM ranges were specified directly.
            flash_len = get_env_var(LINKER_FLASH_LEN_VAR);
            ram_len = get_env_var(LINKER_RAM_LEN_VAR);
            (
                flash_start.deref(),
                flash_len.as_deref().unwrap_or("0xD0000"),
                ram_start.deref(),
                ram_len.as_deref().unwrap_or("46K"),
            )
        }
        (Some(platform), None, None) => {
            // LIBTOCK_PLATFORM was specified.
            match PLATFORMS
                .iter()
                .find(|&&(name, _, _, _, _)| name == platform)
            {
                None => panic!("Unknown platform: {}", platform),
                Some(&(_, flash_start, flash_len, ram_start, ram_len)) => {
                    (flash_start, flash_len, ram_start, ram_len)
                }
            }
        }
        _ => panic!(
            "Must specify either {} or both {} and {}; please see \
                     libtock_build_scripts' documentation for more information.",
            PLATFORM_VAR, LINKER_FLASH_VAR, LINKER_RAM_VAR
        ),
    };
    let tbf_header_size;
    let tbf_header_size = match get_env_var(TBF_HEADER_SIZE_VAR) {
        None => "0x80",
        Some(value) => {
            tbf_header_size = value;
            &tbf_header_size
        }
    };

    // Note: cargo fails if run in a path that is not valid Unicode, so this
    // script doesn't need to handle non-Unicode paths. Also, OUT_DIR cannot be
    // in a location with a newline in it, or we have no way to pass
    // rustc-link-search to cargo.
    let out_dir = &*var("OUT_DIR").expect("Unable to read OUT_DIR");
    assert!(
        !out_dir.contains('\n'),
        "Build path contains a newline, which is unsupported"
    );

    // Create a valid linker file with the specified flash and ram locations.
    //
    // ```
    // TBF_HEADER_SIZE = 0x80;
    // FLASH_START = 0x00040000;
    // FLASH_LENGTH = 0x00040000;
    // RAM_START = 0x20008000;
    // RAM_LENGTH = 62K;
    // INCLUDE libtock_layout.ld
    // ```
    let layout_name = format!("{flash_start}.{flash_len}.{ram_start}.{ram_len}.ld");
    let layout_path: PathBuf = [out_dir, &layout_name].iter().collect();
    let mut layout_file =
        File::create(&layout_path).unwrap_or_else(|e| panic!("Could not open layout file: {}", e));
    writeln!(
        layout_file,
        "\
        TBF_HEADER_SIZE = {tbf_header_size};\n\
        FLASH_START = {flash_start};\n\
        FLASH_LENGTH = {flash_len};\n\
        RAM_START = {ram_start};\n\
        RAM_LENGTH = {ram_len};\n\
        INCLUDE {};",
        LIBTOCK_LAYOUT_NAME
    )
    .expect("Failed to write layout file");
    drop(layout_file);

    // Compile the contents of `libtock_layout.ld` into this library as a
    // string, and copy those contents into out_dir at runtime.
    let libtock_layout_path: PathBuf = [out_dir, LIBTOCK_LAYOUT_NAME].iter().collect();
    let mut libtock_layout_file = File::create(libtock_layout_path)
        .unwrap_or_else(|e| panic!("Could not open {}: {}", LIBTOCK_LAYOUT_NAME, e));
    write!(
        libtock_layout_file,
        "{}",
        include_str!("../libtock_layout.ld")
    )
    .expect("Failed to write libtock_layout.ld");
    drop(libtock_layout_file);

    // Tell rustc which linker script to use and where to find it.
    println!("cargo:rustc-link-arg=-T{}", layout_path.display());
    println!("cargo:rustc-link-search={}", out_dir);

    // Configure the alignment size for the linker. This prevents the linker
    // from assuming very large pages (i.e. 65536 bytes) and unnecessarily
    // inserting additional padding into the output ELF.
    println!("cargo:rustc-link-arg=-zmax-page-size=4096");
}

// Retrieves an environment variable as a String. Returns None if the variable
// is not specified and panics if the variable is not valid Unicode.
fn get_env_var(name: &str) -> Option<String> {
    use std::env::{var, VarError};
    match var(name) {
        Ok(value) => Some(value),
        Err(VarError::NotPresent) => None,
        Err(VarError::NotUnicode(value)) => panic!("Non-Unicode value in {}: {:?}", name, value),
    }
}
