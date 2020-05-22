use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process;

static LAYOUT_FILE_NAME: &str = "layout.ld";
static APP_STACK_SIZE: &str = "APP_STACK_SIZE";
static DEFAULT_STACK_SIZE_BYTES: &str = "2048";

fn main() {
    static PLATFORM_ENV_VAR: &str = "PLATFORM";
    static PLATFORM_FILE_NAME: &str = "platform";
    static APP_HEAP_SIZE: &str = "APP_HEAP_SIZE";
    static KERNEL_HEAP_SIZE: &str = "KERNEL_HEAP_SIZE";

    println!("cargo:rerun-if-env-changed={}", PLATFORM_ENV_VAR);
    println!("cargo:rerun-if-env-changed={}", APP_HEAP_SIZE);
    println!("cargo:rerun-if-env-changed={}", APP_STACK_SIZE);
    println!("cargo:rerun-if-env-changed={}", KERNEL_HEAP_SIZE);
    println!("cargo:rerun-if-changed={}", PLATFORM_FILE_NAME);
    println!("cargo:rerun-if-changed={}", LAYOUT_FILE_NAME);

    let stack_size = read_env_var(APP_STACK_SIZE);
    if stack_size.is_none() {
        println!(
            "cargo:warning=No explicit stack size specified. \
             Using default of 2048 bytes. \
             Set this explicitly using the \
             environment variable {}.
             ",
            APP_STACK_SIZE
        );
    }

    let platform_name =
        read_env_var(PLATFORM_ENV_VAR).or_else(|| read_board_name_from_file(PLATFORM_FILE_NAME));
    if let Some(platform_name) = platform_name {
        println!("cargo:rustc-env={}={}", PLATFORM_ENV_VAR, platform_name);
        create_linker_file(platform_name.trim(), stack_size);
    } else {
        println!(
            "cargo:warning=No platform specified. \
             Remember to manually specify a linker file.",
        );
    }

    set_default_env(APP_HEAP_SIZE, "1024");
    set_default_env(KERNEL_HEAP_SIZE, "1024");
    set_default_env(APP_STACK_SIZE, DEFAULT_STACK_SIZE_BYTES);
}

fn set_default_env(env_var: &str, default: &str) {
    if let Some(s) = read_env_var(env_var) {
        println!("cargo:rustc-env={}={}", env_var, s);
    } else {
        // Use the provided default value if nothing is passed in
        println!("cargo:rustc-env={}={}", env_var, default);
    }
}

fn read_env_var(env_var: &str) -> Option<String> {
    env::var_os(env_var).map(|os_string| os_string.into_string().unwrap())
}

fn read_board_name_from_file(file_name: &str) -> Option<String> {
    let path = Path::new(file_name);
    if !path.exists() {
        return None;
    }

    let board_file = File::open(path).unwrap();
    let mut board_name = String::new();
    BufReader::new(board_file)
        .read_line(&mut board_name)
        .unwrap();
    Some(board_name)
}

fn create_linker_file(platform_name: &str, stack_size: Option<String>) {
    let linker_file_name = format!("boards/layout_{}.ld.handlebars", platform_name);

    let path = Path::new(&linker_file_name);
    if !path.exists() {
        println!("Cannot find layout template file {:?}", path);
        process::exit(1);
    }

    create_linker_file_from_template(
        path,
        &stack_size.unwrap_or_else(|| DEFAULT_STACK_SIZE_BYTES.to_string()),
    )
}

fn create_linker_file_from_template(path: &Path, stack_size: &str) {
    let mut linker_template = File::open(path).expect("Failed to open file.");
    let mut template_content = String::new();
    linker_template
        .read_to_string(&mut template_content)
        .expect("Could not read linke file to string.");

    let processed_template = template_content.replace("{{stack_size}}", &stack_size);

    let out_path = Path::new(LAYOUT_FILE_NAME);
    fs::write(out_path, processed_template).expect("Failed to write linker file.");
}
