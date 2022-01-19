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
    ]);
    // QEMU does something to its stdin that prevents Ctrl+C from generating
    // SIGINT. If we set QEMU's stdin to be our stdin, then Ctrl+C will not
    // close us. To prevent that, we set QEMU's stdin to null.
    qemu.stdin(Stdio::null());
    qemu.stdout(Stdio::piped());
    if cli.verbose {
        println!("QEMU command: {:?}", qemu);
        println!("Spawning QEMU")
    }
    qemu.spawn().expect("failed to spawn QEMU")
}
