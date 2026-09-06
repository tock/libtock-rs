use super::Cli;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

// Spawns a QEMU VM with a simulated Tock system and the process binary. Returns
// the handle for the spawned QEMU process.
pub fn deploy(cli: &Cli, platform: String, tbf_path: PathBuf) -> Child {
    let platform_args = get_platform_args(&platform);
    // clap requires both of these when --deploy is qemu.
    let kernel = cli.kernel.as_ref().expect("--kernel not provided");
    let binary = cli.qemu.as_ref().expect("--qemu not provided");
    let device = format!(
        "loader,file={},addr={}",
        tbf_path
            .into_os_string()
            .into_string()
            .expect("Non-UTF-8 path"),
        platform_args.process_binary_load_address,
    );
    let mut qemu = Command::new(binary);
    qemu.args(["-device", &device, "-nographic", "-serial", "mon:stdio"]);
    qemu.args(platform_args.fixed_args);
    // How the kernel is handed to QEMU differs by board: the virt machine boots
    // the kernel as its firmware, while the others are loaded as a kernel image.
    qemu.args([platform_args.kernel_flag, &kernel.to_string_lossy()]);
    // If we let QEMU inherit its stdin from us, it will set it to raw mode,
    // which prevents Ctrl+C from generating SIGINT. QEMU will not exit when
    // Ctrl+C is entered, making our runner hard to close. Instead, we forward
    // stdin to QEMU ourselves -- see output_processor.rs for more details.
    qemu.stdin(Stdio::piped());
    qemu.stdout(Stdio::piped());
    // Because we set the terminal to raw mode while running QEMU, but QEMU's
    // stdin is not connected to a terminal, QEMU does not know it needs to use
    // CRLF line endings when printing to stderr. To convert, we also pipe
    // QEMU's stderr through us and output_processor converts the line endings.
    qemu.stderr(Stdio::piped());
    if cli.verbose {
        println!("QEMU command: {qemu:?}");
        println!("Spawning QEMU")
    }
    qemu.spawn().unwrap_or_else(|error| {
        panic!(
            "failed to spawn QEMU ({}): {error}\n\
             Install a QEMU with 32-bit RISC-V support (qemu-system-misc on Debian and \
             Ubuntu, qemu-system-riscv on Fedora, qemu on Homebrew), or set LIBTOCK_QEMU \
             to a qemu-system-riscv32 binary.",
            binary.display()
        )
    })
}

// Returns the command line arguments for the given platform to qemu. Panics if
// an unknown platform is passed.
//
// The arguments for each board mirror that board's `qemu` or `run-app` target in
// the Tock tree, which is the authority on how to boot it.
fn get_platform_args(platform: &str) -> PlatformConfig {
    match platform {
        "hifive1" => PlatformConfig {
            fixed_args: &["-M", "sifive_e,revb=true"],
            kernel_flag: "-kernel",
            process_binary_load_address: "0x20040000",
        },
        "opentitan" => PlatformConfig {
            // The earlgrey-cw310 kernel starts at ORIGIN(rom) plus the size of
            // the manifest, so the Ibex has to be told to reset there rather
            // than at the start of the ROM. This replaces a
            // `-bios tock/tools/qemu-runner/opentitan-boot-rom.elf` argument
            // that named a file which does not exist anywhere in Tock.
            #[rustfmt::skip]
            fixed_args: &[
                "-M", "opentitan",
                "-global", "driver=riscv.lowrisc.ibex.soc,property=resetvec,value=0x20000400",
            ],
            kernel_flag: "-kernel",
            process_binary_load_address: "0x20030000",
        },
        "qemu_rv32_virt" => PlatformConfig {
            // The virt board runs the kernel as the machine's firmware, in
            // RISC-V machine mode, rather than as a loaded kernel image.
            #[rustfmt::skip]
            fixed_args: &[
                "-M", "virt",
                "-semihosting",
                "-global", "driver=riscv-cpu,property=smepmp,value=true",
                "-global", "virtio-mmio.force-legacy=false",
                "-device", "virtio-rng-device",
            ],
            kernel_flag: "-bios",
            process_binary_load_address: "0x80100000",
        },
        _ => panic!("Cannot deploy to platform {platform} via QEMU."),
    }
}

// QEMU configuration information that is specific to each platform.
struct PlatformConfig {
    fixed_args: &'static [&'static str],
    // Whether QEMU takes this board's kernel as a firmware image or a kernel
    // image.
    kernel_flag: &'static str,
    process_binary_load_address: &'static str,
}
