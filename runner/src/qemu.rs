use super::Cli;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

// Spawns a QEMU VM with a simulated Tock system and the process binary. Returns
// the handle for the spawned QEMU process.
pub fn deploy(cli: &Cli, platform: String, tbf_path: PathBuf) -> Child {
    let platform_args = get_platform_args(platform);
    let device = format!(
        "loader,file={},addr={}",
        tbf_path
            .into_os_string()
            .into_string()
            .expect("Non-UTF-8 path"),
        platform_args.process_binary_load_address,
    );
    let mut qemu = Command::new("tock/tools/qemu/build/qemu-system-riscv32");
    qemu.args(["-device", &device, "-nographic", "-serial", "mon:stdio"]);
    qemu.args(platform_args.fixed_args);
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
        println!("QEMU command: {:?}", qemu);
        println!("Spawning QEMU")
    }
    qemu.spawn().expect("failed to spawn QEMU")
}

// Returns the command line arguments for the given platform to qemu. Panics if
// an unknown platform is passed.
fn get_platform_args(platform: String) -> PlatformConfig {
    match platform.as_str() {
        "hifive1" => PlatformConfig {
            #[rustfmt::skip]
            fixed_args: &[
                "-kernel", "tock/target/riscv32imac-unknown-none-elf/release/hifive1",
                "-M", "sifive_e,revb=true",
            ],
            process_binary_load_address: "0x20040000",
        },
        "opentitan" => PlatformConfig {
            #[rustfmt::skip]
            fixed_args: &[
                "-bios", "tock/tools/qemu-runner/opentitan-boot-rom.elf",
                "-kernel", "tock/target/riscv32imc-unknown-none-elf/release/earlgrey-cw310",
                "-M", "opentitan",
            ],
            process_binary_load_address: "0x20030000",
        },
        _ => panic!("Cannot deploy to platform {} via QEMU.", platform),
    }
}

// QEMU configuration information that is specific to each platform.
struct PlatformConfig {
    fixed_args: &'static [&'static str],
    process_binary_load_address: &'static str,
}
