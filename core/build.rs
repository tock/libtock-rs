use std::env;

fn main() {
    static APP_HEAP_SIZE: &str = "APP_HEAP_SIZE";

    println!("cargo:rerun-if-env-changed={}", APP_HEAP_SIZE);

    set_default_env(APP_HEAP_SIZE, "1024");
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
