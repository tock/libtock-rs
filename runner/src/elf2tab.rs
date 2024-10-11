use super::Cli;
use std::fs::{metadata, remove_file};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

fn get_platform_architecture(platform: &str) -> Option<&'static str> {
    match platform {
        "raspberry_pi_pico" | "pico_explorer_base" | "nano_rp2040_connect" => Some("cortex-m0"),
        "apollo3"
        | "clue_nrf52840"
        | "hail"
        | "imix"
        | "microbit_v2"
        | "msp432"
        | "nano33ble"
        | "nrf52"
        | "nrf52840"
        | "nucleo_f429zi"
        | "nucleo_f446re"
        | "stm32f3discovery"
        | "stm32f412gdiscovery" => Some("cortex-m4"),
        "imxrt1050" | "teensy40" => Some("cortex-m7"),
        "opentitan" | "esp32_c3_devkitm_1" => Some("riscv32imc"),
        "hifive1" | "qemu_rv32_virt" => Some("riscv32imac"),
        _ => None,
    }
}

// Converts the ELF file specified on the command line into TBF and TAB files,
// and returns the paths to those files.
pub fn convert_elf(cli: &Cli, platform: &str) -> OutFiles {
    let package_name = cli.elf.file_stem().expect("ELF must be a file");
    let mut tab_path = cli.elf.clone();
    tab_path.set_extension("tab");
    if cli.verbose {
        println!("Package name: {:?}", package_name);
        println!("TAB path: {}", tab_path.display());
    }
    let stack_size = read_stack_size(cli);
    let elf = cli.elf.as_os_str();
    let mut tbf_path = cli.elf.clone();
    tbf_path.set_extension("tbf");
    let architecture =
        get_platform_architecture(platform).expect("Failed to determine ELF's architecture");
    if cli.verbose {
        println!("ELF file: {:?}", elf);
        println!("TBF path: {}", tbf_path.display());
    }

    // If elf2tab returns a successful status but does not write to the TBF
    // file, then we run the risk of using an outdated TBF file, creating a
    // hard-to-debug situation. Therefore, we delete the TBF file, forcing
    // elf2tab to create it, and later verify that it exists.
    if let Err(io_error) = remove_file(&tbf_path) {
        // Ignore file-no-found errors, panic on any other error.
        if io_error.kind() != ErrorKind::NotFound {
            panic!("Unable to remove the TBF file. Error: {}", io_error);
        }
    }

    let mut command = Command::new("elf2tab");
    #[rustfmt::skip]
    command.args([
        // TODO: libtock-rs' crates are designed for Tock 2.1's Allow interface,
        // so we should increment this as soon as the Tock kernel will accept a
        // 2.1 app.
        "--kernel-major".as_ref(), "2".as_ref(),
        "--kernel-minor".as_ref(), "0".as_ref(),
        "-n".as_ref(), package_name,
        "-o".as_ref(), tab_path.as_os_str(),
        "--stack".as_ref(), stack_size.as_ref(),
        format!("{},{}", elf.to_str().unwrap(), architecture).as_ref(),
    ]);
    if cli.verbose {
        command.arg("-v");
        println!("elf2tab command: {:?}", command);
        println!("Spawning elf2tab");
    }
    let mut child = command.spawn().expect("failed to spawn elf2tab");
    let status = child.wait().expect("failed to wait for elf2tab");
    if cli.verbose {
        println!("elf2tab finished. {}", status);
    }
    assert!(status.success(), "elf2tab returned an error. {}", status);

    // Verify that elf2tab created the TBF file, and that it is a file.
    match metadata(&tbf_path) {
        Err(io_error) => {
            if io_error.kind() == ErrorKind::NotFound {
                panic!("elf2tab did not create {}", tbf_path.display());
            }
            panic!(
                "Unable to query metadata for {}: {}",
                tbf_path.display(),
                io_error
            );
        }
        Ok(metadata) => {
            assert!(metadata.is_file(), "{} is not a file", tbf_path.display());
        }
    }

    OutFiles { tab_path, tbf_path }
}

// Paths to the files output by elf2tab.
pub struct OutFiles {
    pub tab_path: PathBuf,
    pub tbf_path: PathBuf,
}

// Reads the stack size, and returns it as a String for use on elf2tab's command
// line.
fn read_stack_size(cli: &Cli) -> String {
    let file = elf::File::open_path(&cli.elf).expect("Unable to open ELF");
    for section in file.sections {
        // This section name comes from runtime/libtock_layout.ld, and it
        // matches the size (and location) of the process binary's stack.
        if section.shdr.name == ".stack" {
            let stack_size = section.shdr.size.to_string();
            if cli.verbose {
                println!("Found .stack section, size: {}", stack_size);
            }
            return stack_size;
        }
    }

    panic!("Unable to find the .stack section in {}", cli.elf.display());
}
