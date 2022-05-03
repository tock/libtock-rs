use super::Cli;
use libc::{kill, pid_t, SIGINT};
use std::io::{stderr, stdin, stdout, BufRead, BufReader, ErrorKind, Stdout, Write};
use std::process::Child;
use std::thread::spawn;
use termion::raw::{IntoRawMode, RawTerminal};

/// Reads the console messages from `child`'s standard output, sending SIGTERM
/// to the child when the process is terminated.
pub fn process(cli: &Cli, mut child: Child) {
    let raw_mode = forward_stdin_if_piped(&mut child);
    forward_stderr_if_piped(&mut child, raw_mode.is_some());
    let mut to_print = Vec::new();
    let mut reader = BufReader::new(child.stdout.as_mut().expect("Child's stdout not piped."));
    loop {
        let buffer = reader
            .fill_buf()
            .expect("Unable to read from child process.");
        if buffer.is_empty() {
            // The child process has closed its stdout, likely by exiting.
            break;
        }
        // Print the bytes received over stdout. If the terminal is in raw mode,
        // translate '\n' into '\r\n'.
        for &byte in buffer {
            if raw_mode.is_some() && byte == b'\n' {
                to_print.push(b'\r');
            }
            to_print.push(byte);
        }
        let stdout = stdout();
        let mut lock = stdout.lock();
        lock.write_all(&to_print)
            .expect("Unable to echo child's stdout.");
        let _ = lock.flush();
        drop(lock);
        to_print.clear();

        let buffer_len = buffer.len();
        reader.consume(buffer_len);
    }
    if cli.verbose {
        println!("Waiting for child process.\r");
    }
    let status = child.wait().expect("Unable to wait for child process");
    drop(raw_mode);
    assert!(
        status.success(),
        "Child process did not exit successfully. {}",
        status
    );
}

// If child's stdin is piped, this sets the terminal to raw mode and spawns a
// thread that forwards our stdin to child's stdin. The thread sends SIGINT to
// the child if Ctrl+C is pressed. Returns a RawTerminal, which reverts the
// terminal to its previous configuration on drop.
fn forward_stdin_if_piped(child: &mut Child) -> Option<RawTerminal<Stdout>> {
    let mut child_stdin = child.stdin.take()?;
    let child_id = child.id();
    spawn(move || {
        let our_stdin = stdin();
        let mut our_stdin = our_stdin.lock();
        loop {
            let buffer = our_stdin.fill_buf().expect("Failed to read stdin.");
            if buffer.is_empty() {
                // Our stdin was closed. We interpret this as a signal to exit,
                // because pressing Ctrl+C to trigger an exit is no longer
                // possible.
                break;
            }
            // In raw mode, pressing Ctrl+C will send a '3' byte to stdin ("end
            // of message" ASCII value) instead of sending SIGINT. Identify that
            // case, and exit if it occurs.
            if buffer.contains(&3) {
                break;
            }
            match child_stdin.write(buffer) {
                // A BrokenPipe error occurs when the child has exited. Exit
                // without sending SIGINT.
                Err(error) if error.kind() == ErrorKind::BrokenPipe => return,

                Err(error) => panic!("Failed to forward stdin: {}", error),
                Ok(bytes) => our_stdin.consume(bytes),
            }
        }
        // Send SIGINT to the child, telling it to exit. After the child exits,
        // the main loop will detect the exit and we will shut down cleanly.
        //
        // Safety: Sending SIGINT to a process is a safe operation -- kill is
        // marked unsafe because it is a FFI function.
        unsafe {
            kill(child_id as pid_t, SIGINT);
        }
    });
    Some(
        stdout()
            .into_raw_mode()
            .expect("Failed to set terminal to raw mode."),
    )
}

// Forwards child's stderr to our stderr if child's stderr is piped, converting
// line endings to CRLF if raw_mode is true.
fn forward_stderr_if_piped(child: &mut Child, raw_mode: bool) {
    let child_stderr = match child.stderr.take() {
        None => return,
        Some(child_stderr) => child_stderr,
    };
    spawn(move || {
        let mut to_print = Vec::new();
        let mut reader = BufReader::new(child_stderr);
        loop {
            let buffer = reader.fill_buf().expect("Unable to read child's stderr.");
            if buffer.is_empty() {
                return;
            }
            for &byte in buffer {
                if raw_mode && byte == b'\n' {
                    to_print.push(b'\r');
                }
                to_print.push(byte);
            }
            stderr()
                .write_all(&to_print)
                .expect("Unable to echo child's stderr.");
            to_print.clear();
            let buffer_len = buffer.len();
            reader.consume(buffer_len);
        }
    });
}
