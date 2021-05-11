//! Build script module for compiling the external assembly (used for the entry
//! point) and linking it into the process binary. Requires out_dir to be added
//! to rustc's link search path.

/// Compiles the external assembly and tells cargo/rustc to link the resulting
/// library into the crate. Panics if it is unable to find a working assembly
/// toolchain or if the assembly fails to compile.
pub(crate) fn build_and_link(out_dir: &str) {
    use std::env::var;
    let arch = var("CARGO_CFG_TARGET_ARCH").expect("Unable to read CARGO_CFG_TARGET_ARCH");

    // Identify the toolchain configurations to try for the target architecture.
    // We support trying multiple toolchains because not all toolchains are
    // available on every OS that we want to support development on.
    let build_configs: &[AsmBuildConfig] = match arch.as_str() {
        "arm" => &[AsmBuildConfig {
            prefix: "arm-none-eabi",
            as_extra_args: &[],
            strip: false,
        }],
        "riscv32" => &[
            // First try riscv64-unknown-elf, as it is the toolchain used by
            // libtock-c and the toolchain used in the CI environment.
            AsmBuildConfig {
                prefix: "riscv64-unknown-elf",
                as_extra_args: &["-march=rv32imc"],
                strip: true,
            },
            // Second try riscv32-unknown-elf. This is the best match for Tock's
            // risc-v targets, but is not as widely available (and has not been
            // tested with libtock-rs yet).
            AsmBuildConfig {
                prefix: "riscv32-unknown-elf",
                as_extra_args: &[],
                strip: false, // Untested, may need to change.
            },
            // Last try riscv64-linux-gnu, as it is the only option on Debian 10
            AsmBuildConfig {
                prefix: "riscv64-linux-gnu",
                as_extra_args: &["-march=rv32imc"],
                strip: true,
            },
        ],
        unknown_arch => {
            panic!("Unsupported architecture {}", unknown_arch);
        }
    };

    // Loop through toolchain configs until one works.
    for &build_config in build_configs {
        if try_build(&arch, build_config, out_dir).is_ok() {
            return;
        }
    }

    panic!("Unable to find a toolchain for architecture {}", arch);
}

#[derive(Clone, Copy)]
struct AsmBuildConfig {
    // Prefix, which is prepended to the command names.
    prefix: &'static str,

    // Extra arguments to pass to the assembler.
    as_extra_args: &'static [&'static str],

    // Do we need to strip the object file before packing it into the library
    // archive? This should be set to true on platforms where the assembler adds
    // local symbols to the object file.
    strip: bool,
}

// Indicates the toolchain in the build config is unavailable.
struct ToolchainUnavailable;

fn try_build(
    arch: &str,
    build_config: AsmBuildConfig,
    out_dir: &str,
) -> Result<(), ToolchainUnavailable> {
    use std::path::PathBuf;
    use std::process::Command;

    // Invoke the assembler to produce an object file.
    let asm_source = &format!("asm/asm_{}.S", arch);
    let obj_file_path = [out_dir, "libtock_rt_asm.o"].iter().collect::<PathBuf>();
    let obj_file = obj_file_path.to_str().expect("Non-Unicode obj_file_path");
    let as_result = Command::new(format!("{}-as", build_config.prefix))
        .args(build_config.as_extra_args)
        .args(&[asm_source, "-o", obj_file])
        .status();

    match as_result {
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                // This `as` command does not exist. Return an error so
                // build_an_link can try another config (if one is available).
                return Err(ToolchainUnavailable);
            } else {
                panic!("Error invoking assembler: {}", error);
            }
        }
        Ok(status) => {
            assert!(status.success(), "Assembler returned an error");
        }
    }

    // At this point, we know this toolchain is installed. We will fail if later
    // commands are uninstalled rather than trying a different build config.

    println!("cargo:rerun-if-changed={}", asm_source);

    // Run `strip` if necessary.
    if build_config.strip {
        let strip_cmd = format!("{}-strip", build_config.prefix);
        let status = Command::new(&strip_cmd)
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
        if error.kind() != std::io::ErrorKind::NotFound {
            panic!("Unable to remove archive file {}", archive_path.display());
        }
    }

    // Create the library archive.
    let ar_cmd = format!("{}-ar", build_config.prefix);
    let archive = archive_path.to_str().expect("Non-Unicode archive_path");
    let status = std::process::Command::new(&ar_cmd)
        // c == Do not complain if archive needs to be created.
        // r == Insert or replace file in archive.
        .args(&["cr", archive, obj_file])
        .status()
        .unwrap_or_else(|_| panic!("Failed to invoke {}", ar_cmd));
    assert!(status.success(), "{} returned an error", ar_cmd);

    // Tell rustc to link the binary against the library archive.
    println!("cargo:rustc-link-lib=static={}", ARCHIVE_NAME);

    Ok(())
}
