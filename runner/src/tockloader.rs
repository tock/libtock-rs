use super::Cli;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

// Uses tockloader to deploy the provided TAB file to a Tock system. Returns the
// handle for the spawned 'tockloader listen' process.
// Note: This function is untested, as its author does not have hardware that
// works with tockloader. If you use it, please report back on how it works so
// we can fix it or remove this notice!
pub fn deploy(cli: &Cli, platform: String, tab_path: PathBuf) -> Child {
    let flags: &[_] = match platform.as_str() {
        "clue_nrf52840" => &[],
        "hail" | "imix" => &[],
        "microbit_v2" => &["--bundle-apps"],
        "nrf52" | "nrf52840" => &[
            "--jlink",
            "--arch",
            "cortex-m4",
            "--board",
            "nrf52dk",
            "--jtag-device",
            "nrf52",
        ],
        _ => panic!("Cannot deploy to platform {} via tockloader", platform),
    };
    if cli.verbose {
        println!("Tockloader flags: {:?}", flags);
    }

    // Tockloader listen's ability to receive every message from the Tock system
    // varies from platform to platform. We look up the platform, and if it is
    // not satisfactorily reliable we output a warning for the user.
    let reliable_listen = match platform.as_str() {
        // tockloader listen will reset the Hail/Imix, allowing it to capture all
        // printed messages.
        "hail" | "imix" => true,

        // Microbit uses CDC over USB, which buffers messages so that tockloader
        // listen can receive messages sent before it was started. As long as
        // tockloader listen launches before the timeout, there will not be
        // dropped messages. This is good enough for our purposes.
        "microbit_v2" => true,

        // tockloader listen doesn't reset the nrf52, and there's no message
        // queueing mechanism. Therefore, tockloader listen will likely miss
        // messages printed quickly after the process binary is deployed.
        "nrf52" | "nrf52840" => false,

        // We shouldn't hit this case, because the flag determination code above
        // should error out on unknown platforms.
        _ => panic!("Unknown reliability for {}", platform),
    };
    if !reliable_listen {
        println!(
            "Warning: tockloader listen may miss early messages on platform {}",
            platform
        );
    }

    // Invoke tockloader uninstall to remove the process binary, if present.
    let mut uninstall = Command::new("tockloader");
    uninstall.arg("uninstall");
    uninstall.args(flags);
    if cli.verbose {
        println!("tockloader uninstall command: {:?}", uninstall);
    }
    let mut child = uninstall
        .spawn()
        .expect("failed to spawn tockloader uninstall");
    let status = child
        .wait()
        .expect("failed to wait for tockloader uninstall");
    if cli.verbose {
        println!("tockloader uninstall finished. {}", status);
    }

    // Invoke tockloader install to deploy the new process binary.
    let mut install = Command::new("tockloader");
    install.arg("install");
    install.args(flags);
    install.arg(tab_path);
    if cli.verbose {
        println!("tockloader install command: {:?}", install);
    }
    let mut child = install.spawn().expect("failed to spawn tockloader install");
    let status = child.wait().expect("failed to wait for tockloader install");
    if cli.verbose {
        println!("tockloader install finished. {}", status);
    }
    assert!(
        status.success(),
        "tockloader install returned unsuccessful status {}",
        status
    );

    // Invoke tockloader listen to receive messages from the Tock system.
    let mut listen = Command::new("tockloader");
    listen.arg("listen");
    listen.args(flags);
    listen.stdout(Stdio::piped());
    if cli.verbose {
        println!("tockloader listen command: {:?}", listen);
    }
    listen.spawn().expect("failed to spawn tockloader listen")
}
