//! Build script module for compiling the external assembly (used for the entry
//! point) and linking it into the process binary. Requires out_dir to be added
//! to rustc's link search path.

use std::{
    env, io,
    process::{self, Command},
};

#[derive(Clone)]
struct ToolchainInfo {
    arch: String,

    /// The library archive command.
    ar_cmd: String,

    /// The object file strip command, if used.
    /// If `None`, no strip will take place.
    strip_cmd: Option<String>,

    /// The assembler command.
    as_cmd: String,

    /// Additional flags to pass to the assembler.
    as_flags: Vec<String>,
}

impl ToolchainInfo {
    fn from_gcc_prefix(arch: &str, prefix: &str, strip: bool) -> Self {
        Self {
            arch: arch.to_string(),
            ar_cmd: format!("{}-ar", prefix),
            strip_cmd: strip.then(|| format!("{}-strip", prefix)),
            as_cmd: format!("{}-as", prefix),
            as_flags: Vec::new(),
        }
    }
}

/// Reads toolchain info from local environment variables, merging with a
/// default toolchain if provided.
///
/// If a complete toolchain could not be constructed, this returns `None`.
fn env_toolchain(arch: &str, default: Option<ToolchainInfo>) -> Option<ToolchainInfo> {
    let env_ar_cmd = env::var("AR").ok();
    let env_as_cmd = env::var("AS").ok();
    // TODO: this cannot handle whitespace in flags.
    let env_as_flags = env::var("ASFLAGS")
        .ok()
        .map(|x| x.split_ascii_whitespace().map(str::to_string).collect());
    let env_strip_cmd = env::var("STRIP").ok().filter(|x| !x.is_empty());

    let ar_cmd;
    let as_cmd;
    let as_flags;
    let strip_cmd;
    if let Some(default) = default {
        // TODO: Should this merging occur or be all-or-nothing with
        //       environment-provided toolchain variables?
        ar_cmd = env_ar_cmd.unwrap_or(default.ar_cmd);
        as_cmd = env_as_cmd.unwrap_or(default.as_cmd);
        as_flags = env_as_flags.unwrap_or(default.as_flags);
        strip_cmd = env_strip_cmd.or(default.strip_cmd);
    } else {
        ar_cmd = env_ar_cmd?;
        as_cmd = env_as_cmd?;
        as_flags = env_as_flags.unwrap_or(Vec::new());
        strip_cmd = env_strip_cmd;
    }

    Some(ToolchainInfo {
        arch: arch.to_string(),
        ar_cmd,
        strip_cmd,
        as_cmd,
        as_flags,
    })
}

/// Checks a toolchain, by running `--version` on the provided commands.
fn test_toolchain(toolchain: &ToolchainInfo) -> io::Result<()> {
    Command::new(&toolchain.ar_cmd).arg("--version").status()?;
    Command::new(&toolchain.as_cmd).arg("--version").status()?;
    if let Some(ref path) = toolchain.strip_cmd {
        Command::new(path).arg("--version").status()?;
    }
    Ok(())
}

fn find_default_toolchain(arch: &str) -> Option<ToolchainInfo> {
    // The default toolchain is the first GCC-like toolchain that works as
    // expected. We support trying multiple toolchains because not all
    // toolchains are available on every OS that we want to support development.
    match arch {
        "arm" => {
            let toolchain = ToolchainInfo::from_gcc_prefix(arch, "arm-none-eabi", false);
            test_toolchain(&toolchain).ok().map(|_| toolchain)
        }
        "riscv32" => {
            for toolchain in [
                ToolchainInfo {
                    as_flags: vec!["-march=rv32imc".to_string()],
                    ..ToolchainInfo::from_gcc_prefix(arch, "riscv64-unknown-elf", true)
                },
                // strip: false is untested here, may need to change
                ToolchainInfo::from_gcc_prefix(arch, "riscv32-unknown-elf", false),
                ToolchainInfo {
                    as_flags: vec!["-march=rv32imc".to_string()],
                    ..ToolchainInfo::from_gcc_prefix(arch, "riscv64-linux-gnu", true)
                },
            ] {
                if test_toolchain(&toolchain).is_ok() {
                    return Some(toolchain);
                }
            }
            None
        }
        unknown_arch => {
            panic!("Unsupported architecture {}", unknown_arch)
        }
    }
}

/// Compiles the external assembly and tells cargo/rustc to link the resulting
/// library into the crate. Panics if it is unable to find a working assembly
/// toolchain or if the assembly fails to compile.
pub(crate) fn build_and_link(out_dir: &str) {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("Unable to read CARGO_CFG_TARGET_ARCH");

    match env_toolchain(&arch, find_default_toolchain(&arch)) {
        Some(toolchain) => build(&toolchain, out_dir),
        None => panic!("Unable to find a toolchain for architecture {}", arch),
    }
}

fn build(toolchain: &ToolchainInfo, out_dir: &str) {
    use std::path::PathBuf;

    // Invoke the assembler to produce an object file.
    let as_cmd = &toolchain.as_cmd;
    let asm_source = &format!("asm/asm_{}.S", toolchain.arch);
    let obj_file_path = [out_dir, "libtock_rt_asm.o"].iter().collect::<PathBuf>();
    let obj_file = obj_file_path.to_str().expect("Non-Unicode obj_file_path");
    let status = Command::new(as_cmd)
        .args(&toolchain.as_flags)
        .args(&[asm_source, "-o", obj_file])
        .status()
        .unwrap_or_else(|_| panic!("Failed to invoke {}", as_cmd));
    assert!(status.success(), "{} returned an error", as_cmd);

    println!("cargo:rerun-if-changed={}", asm_source);

    // Run `strip` if necessary.
    if let Some(strip_cmd) = &toolchain.strip_cmd {
        let status = Command::new(strip_cmd)
            .args(&["-K", "start", "-K", "rust_start", obj_file])
            .status()
            .unwrap_or_else(|_| panic!("Failed to invoke {}", strip_cmd));
        assert!(status.success(), "{} returned an error", strip_cmd);
    }

    // Remove the archive file in case there is something unexpected in it. This
    // prevents issues from persisting across invocations of this script.
    const ARCHIVE_NAME: &str = "tock_rt_asm";
    let archive_path: PathBuf = [out_dir, &format!("lib{}.a", ARCHIVE_NAME)]
        .iter()
        .collect();
    if let Err(error) = std::fs::remove_file(&archive_path) {
        if error.kind() != io::ErrorKind::NotFound {
            panic!("Unable to remove archive file {}", archive_path.display());
        }
    }

    // Create the library archive.
    let ar_cmd = &toolchain.ar_cmd;
    let archive = archive_path.to_str().expect("Non-Unicode archive_path");
    let status = process::Command::new(ar_cmd)
        // c == Do not complain if archive needs to be created.
        // r == Insert or replace file in archive.
        .args(&["cr", archive, obj_file])
        .status()
        .unwrap_or_else(|_| panic!("Failed to invoke {}", ar_cmd));
    assert!(status.success(), "{} returned an error", ar_cmd);

    // Tell rustc to link the binary against the library archive.
    println!("cargo:rustc-link-lib=static={}", ARCHIVE_NAME);
}
