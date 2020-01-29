use std::env;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process;

fn main() {
    static ENV_VAR: &str = "PLATFORM";
    static FILE_NAME: &str = "platform";

    let platform_name = read_board_name_from_env_var(ENV_VAR)
        .or_else(|| read_board_name_from_file(FILE_NAME))
        .unwrap_or_else(|| {
            println!(
                "No platform specified. Either set the environment \
                 variable {} or create a file named `{}`",
                ENV_VAR, FILE_NAME
            );
            process::exit(1);
        });

    println!("cargo:rustc-env={}={}", ENV_VAR, platform_name);

    copy_linker_file(&platform_name.trim());
}

fn read_board_name_from_env_var(env_var: &str) -> Option<String> {
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

fn copy_linker_file(platform_name: &str) {
    let linker_file_name = format!("layout_{}.ld", platform_name);
    let path = Path::new(&linker_file_name);
    if !path.exists() {
        println!("Cannot find layout file {:?}", path);
        process::exit(1);
    }
    fs::copy(linker_file_name, "layout.ld").unwrap();
}
