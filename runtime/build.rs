//! This build.rs makes common linker scripts available when linking libtock-rs
//! apps.

fn main() {
    use std::fs::copy;
    use std::fs::read_dir;
    use std::path::PathBuf;

    const LAYOUT_GENERIC_FILENAME: &str = "libtock_layout.ld";

    // Note: cargo fails if run in a path that is not valid Unicode, so this
    // script doesn't need to handle non-Unicode paths. Also, OUT_DIR cannot be
    // in a location with a newline in it, or we have no way to pass
    // rustc-link-search to cargo.
    let out_dir = &std::env::var("OUT_DIR").expect("Unable to read OUT_DIR");
    assert!(
        !out_dir.contains('\n'),
        "Build path contains a newline, which is unsupported"
    );

    // Copy all platform linker scripts to a directory where the linker can find
    // them.
    let paths = read_dir("layouts").unwrap();
    for path in paths {
        let ld_path = path.as_ref().unwrap().path();
        let ld_name = path.as_ref().unwrap().file_name().into_string().unwrap();
        let out_ld_path: PathBuf = [out_dir, &ld_name].iter().collect();
        copy(ld_path, out_ld_path).expect("Unable to copy platform layout into OUT_DIR");
    }

    // Copy the generic layout file into OUT_DIR.
    let out_layout_generic: PathBuf = [out_dir, LAYOUT_GENERIC_FILENAME].iter().collect();
    println!("cargo:rerun-if-changed={}", LAYOUT_GENERIC_FILENAME);
    copy(LAYOUT_GENERIC_FILENAME, out_layout_generic)
        .expect("Unable to copy layout_generic.ld into OUT_DIR");

    // Tell the linker that it can find linker scripts in the out directory of
    // the libtock_runtime crate.
    println!("cargo:rustc-link-search={}", out_dir);
}
