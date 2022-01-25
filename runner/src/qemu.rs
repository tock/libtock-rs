use super::Cli;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

// Spawns a QEMU VM with a simulated Tock system and the process binary. Returns
// the handle for the spawned QEMU process.
pub fn deploy(cli: &Cli, tab_path: PathBuf) -> Child {
    let device = format!(
        "loader,file={},addr=0x20040000",
        tab_path
            .into_os_string()
            .into_string()
            .expect("Non-UTF-8 path")
    );
    let mut qemu = Command::new("tock2/tools/qemu/build/qemu-system-riscv32");
    #[rustfmt::skip]
    qemu.args([
        "-device", &device,
        "-kernel", "tock2/target/riscv32imac-unknown-none-elf/release/hifive1",
        "-M", "sifive_e,revb=true",
        "-nographic",
        "-serial", "mon:stdio",
    ]);
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
