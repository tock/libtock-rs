use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

static LAYOUT_FILE_NAME: &str = "layout.ld";
static LAYOUT_GENERIC_FILENAME: &str = "layout_generic.ld";

fn main() {
    static PLATFORM_ENV_VAR: &str = "PLATFORM";
    static APP_HEAP_SIZE: &str = "APP_HEAP_SIZE";
    static KERNEL_HEAP_SIZE: &str = "KERNEL_HEAP_SIZE";

    println!("cargo:rerun-if-env-changed={}", PLATFORM_ENV_VAR);
    println!("cargo:rerun-if-env-changed={}", APP_HEAP_SIZE);
    println!("cargo:rerun-if-env-changed={}", KERNEL_HEAP_SIZE);
    println!("cargo:rerun-if-changed={}", LAYOUT_FILE_NAME);
    println!("cargo:rerun-if-changed={}", LAYOUT_GENERIC_FILENAME);

    let platform_name = read_env_var(PLATFORM_ENV_VAR);
    if let Some(platform_name) = platform_name {
        println!("cargo:rustc-env={}={}", PLATFORM_ENV_VAR, platform_name);
        copy_linker_file(platform_name.trim());
    } else {
        println!(
            "cargo:warning=No platform specified. \
             Remember to manually specify a linker file.",
        );
    }

    set_default_env(APP_HEAP_SIZE, "1024");
    set_default_env(KERNEL_HEAP_SIZE, "1024");
}

fn set_default_env(env_var: &str, default: &str) {
    if let Some(s) = read_env_var(env_var) {
        println!("cargo:rustc-env={}={}", env_var, s);
    } else {
        // Just use a default of 1024 if nothing is passed in
        println!("cargo:rustc-env={}={}", env_var, default);
    }
}

fn read_env_var(env_var: &str) -> Option<String> {
    env::var_os(env_var).map(|os_string| os_string.into_string().unwrap())
}

fn copy_linker_file(platform_name: &str) {
    let linker_file_name = format!("boards/layout_{}.ld", platform_name);
    let path = Path::new(&linker_file_name);
    if !path.exists() {
        println!("Cannot find layout file {:?}", path);
        process::exit(1);
    }
    // Note: cargo fails if run in a path that is not valid Unicode, so this
    // script doesn't need to handle non-Unicode paths. Also, OUT_DIR cannot be
    // in a location with a newline in it, or we have no way to pass
    // rustc-link-search to cargo.
    let out_dir = &std::env::var("OUT_DIR").expect("Unable to read OUT_DIR");
    assert!(
        !out_dir.contains('\n'),
        "Build path contains a newline, which is unsupported"
    );
    let out_layout_path: PathBuf = [out_dir, "layout.ld"].iter().collect();
    fs::copy(linker_file_name, out_layout_path).unwrap();

    // Copy the generic layout file into OUT_DIR.
    let out_layout_generic: PathBuf = [out_dir, LAYOUT_GENERIC_FILENAME].iter().collect();
    fs::copy(LAYOUT_GENERIC_FILENAME, out_layout_generic)
        .expect("Unable to copy layout_generic.ld into OUT_DIR");
    println!("cargo:rustc-link-search={}", out_dir);
}
