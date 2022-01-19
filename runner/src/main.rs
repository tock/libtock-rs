mod elf2tab;
mod output_processor;
mod qemu;
mod tockloader;

use clap::{ArgEnum, Parser};
use std::path::PathBuf;

/// Converts ELF binaries into Tock Binary Format binaries and runs them on a
/// Tock system.
#[derive(Debug, Parser)]
pub struct Cli {
    /// Where to deploy the process binary. If not specified, runner will only
    /// make a TBF file and not attempt to run it.
    #[clap(arg_enum, long, short)]
    deploy: Option<Deploy>,

    /// The executable to convert into Tock Binary Format and run.
    elf: PathBuf,

    /// Whether to output verbose debugging information to the console.
    #[clap(long, short)]
    verbose: bool,
}

#[derive(ArgEnum, Clone, Debug)]
pub enum Deploy {
    Qemu,
    Tockloader,
}

fn main() {
    let cli = Cli::parse();
    let paths = elf2tab::convert_elf(&cli);
    let child = match cli.deploy {
        None => return,
        Some(Deploy::Qemu) => qemu::deploy(&cli, paths.tbf_path),
        Some(Deploy::Tockloader) => tockloader::deploy(&cli, paths.tab_path),
    };
    output_processor::process(&cli, child);
}
