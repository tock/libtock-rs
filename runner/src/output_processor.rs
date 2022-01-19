use super::Cli;
use signal_hook::flag::{register, register_conditional_default};
use std::io::{stdout, BufReader, Read, Write};
use std::process::Child;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Reads the console messages from `child`'s standard output, sending SIGTERM
/// to the child when the process is terminated.
pub fn process(cli: &Cli, mut child: Child) {
    // When Ctrl+C is pressed, Bash sends SIGINT to both us and our child
    // process. If we finish before the child process does, then bash will print
    // the shell prompt before the child process prints its final messages,
    // which is annoying for the user.
    //
    // So instead, we ignore the first SIGINT we receive. Ctrl+C should
    // therefore result in a clean shutdown: the child process exits, then we
    // see that it finished and terminates. As a fail-safe in case we fail to
    // terminate, this combination will make us exit if SIGINT is received a
    // second time.
    let sigint_received = Arc::new(AtomicBool::new(false));
    register_conditional_default(signal_hook::consts::SIGINT, sigint_received.clone())
        .expect("Unable to register SIGINT conditional handler.");
    register(signal_hook::consts::SIGINT, sigint_received)
        .expect("Unable to register SIGINT handler.");

    let reader = BufReader::new(child.stdout.as_mut().expect("Child's stdout not piped."));
    for byte in reader.bytes() {
        let byte = byte.expect("Unexpected IO error.");
        stdout()
            .write_all(&[byte])
            .expect("Failed to write to stdout.");
    }
    if cli.verbose {
        println!("Waiting for child process to exit");
    }
    let status = child.wait().expect("Unable to wait for child process");
    assert!(
        status.success(),
        "Child process did not exit successfully. {}",
        status
    );
}
