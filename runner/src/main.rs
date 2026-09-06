mod elf2tab;
mod output_processor;
mod qemu;
mod tockloader;

use clap::builder::NonEmptyStringValueParser;
use clap::{Parser, ValueEnum};
use std::env::{var, VarError};
use std::path::PathBuf;

/// Converts ELF binaries into Tock Binary Format binaries and runs them on a
/// Tock system.
#[derive(Debug, Parser)]
pub struct Cli {
    /// Where to deploy the process binary. If not specified, runner will only
    /// make a TBF file and not attempt to run it.
    #[clap(action, long, short, value_enum)]
    deploy: Option<Deploy>,

    /// The executable to convert into Tock Binary Format and run.
    #[clap(action)]
    elf: PathBuf,

    /// Shut the system down and report success as soon as this string
    /// appears in its output.
    #[clap(action, long, value_parser = NonEmptyStringValueParser::new())]
    expect: Option<String>,

    /// The Tock kernel to boot, when deploying to QEMU.
    #[clap(action, long, required_if_eq("deploy", "qemu"))]
    kernel: Option<PathBuf>,

    /// The QEMU binary to run, when deploying to QEMU.
    #[clap(action, long, required_if_eq("deploy", "qemu"))]
    qemu: Option<PathBuf>,

    /// Terminate and fail if the system has not exited after this many seconds.
    #[clap(action, long)]
    timeout: Option<u64>,

    /// Whether to output verbose debugging information to the console.
    #[clap(long, short, action)]
    verbose: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Deploy {
    Qemu,
    Tockloader,
}

fn main() {
    let cli = Cli::parse();
    let platform = match var("LIBTOCK_PLATFORM") {
        Err(VarError::NotPresent) => {
            panic!("LIBTOCK_PLATFORM must be specified to deploy")
        }
        Err(VarError::NotUnicode(platform)) => {
            panic!("Non-UTF-8 LIBTOCK_PLATFORM value: {platform:?}")
        }
        Ok(platform) => platform,
    };
    if cli.verbose {
        println!("Detected platform {platform}");
    }
    if matches!(cli.deploy, Some(Deploy::Qemu)) {
        output_processor::check_run_can_end(&cli);
    }
    let paths = elf2tab::convert_elf(&cli, &platform);
    let deploy = match cli.deploy {
        None => return,
        Some(deploy) => deploy,
    };
    let child = match deploy {
        Deploy::Qemu => qemu::deploy(&cli, platform, paths.tbf_path),
        Deploy::Tockloader => tockloader::deploy(&cli, platform, paths.tab_path),
    };
    output_processor::process(&cli, child);
}
