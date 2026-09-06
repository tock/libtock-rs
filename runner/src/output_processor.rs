use super::Cli;
use std::io::{stderr, stdin, stdout, BufRead, BufReader, ErrorKind, Stdout, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{exit, Child, ChildStderr, ChildStdin};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;
use termion::is_tty;
use termion::raw::{IntoRawMode, RawTerminal};

// The signal Child::kill sends. Rather than depend on libc for one constant, we
// name the number POSIX fixes it at.
const SIGKILL: i32 = 9;

// How long the timeout thread gives the rest of the runner to finish reporting
// the timeout before it ends the process itself.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Panics unless something would eventually end an emulated run.
///
/// An emulated board runs until it is interrupted, and the only thing that
/// interrupts one is a person pressing Ctrl+C. Without a terminal there is no
/// such person, so the run needs --expect or --timeout to say when it is over.
/// Deciding this before the board is started keeps a misconfigured run from
/// booting one only to shut it down before it has printed anything -- which,
/// being a shutdown we asked for, would be reported as a success.
pub fn check_run_can_end(cli: &Cli) {
    assert!(
        interactive() || cli.expect.is_some() || cli.timeout.is_some(),
        "Nothing would end this run: it has no terminal to interrupt it, and \
         neither --expect nor --timeout was given."
    );
}

// Whether there is a person at both ends of this run: a terminal we can
// configure, and a stdin they can type at.
fn interactive() -> bool {
    is_tty(&stdin()) && is_tty(&stdout())
}

/// Reads the console messages from `child`'s standard output, shutting the
/// child down when the process is terminated.
pub fn process(cli: &Cli, mut child: Child) {
    // The child's pipes are taken up front, so that reading them does not
    // require the lock below. The threads that shut the child down need the
    // Child itself, and the main loop spends nearly all of its time blocked
    // reading the child's output, so sharing the Child through a mutex it does
    // not hold while reading is what lets a shutdown arrive at any time.
    let child_stdin = child.stdin.take();
    let child_stdout = child.stdout.take().expect("Child's stdout not piped.");
    let child_stderr = child.stderr.take();
    // Set when we ask the child to shut down, so that we can tell the shutdown
    // we asked for apart from the child dying on its own.
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let child = Arc::new(Mutex::new(child));

    // Set when the timeout elapses, so that we can tell a run we cut short apart
    // from one that ended the way it was asked to.
    let timed_out = Arc::new(AtomicBool::new(false));
    // A run that says how it ends -- with --expect, or at worst with --timeout
    // -- does not need a person at the keyboard, and must not be cut short when
    // our stdin turns out to be empty.
    let ends_on_its_own = cli.expect.is_some() || cli.timeout.is_some();

    let raw_mode = forward_stdin_if_piped(
        child_stdin,
        child.clone(),
        shutdown_requested.clone(),
        ends_on_its_own,
    );
    if let Some(timeout) = cli.timeout {
        shut_down_after_timeout(
            child.clone(),
            shutdown_requested.clone(),
            timed_out.clone(),
            timeout,
        );
    }
    forward_stderr_if_piped(child_stderr, raw_mode.is_some());
    let mut expected = cli.expect.as_deref().map(Expected::new);
    let mut expectation_met = false;
    let mut to_print = Vec::new();
    let mut reader = BufReader::new(child_stdout);
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

        // The output we were waiting for has arrived. Stop the run.
        if let Some(expected) = &mut expected {
            if expected.seen_in(buffer) {
                expectation_met = true;
                request_shutdown(&child, &shutdown_requested);
                break;
            }
        }

        let buffer_len = buffer.len();
        reader.consume(buffer_len);
    }
    if cli.verbose {
        println!("Waiting for child process.\r");
    }
    // wait() would hold the child mutex for as long as the child runs, which
    // would block the threads whose whole job is to end it. Poll instead, so
    // that the lock is only ever held for the length of one try_wait.
    let status = loop {
        let waited = lock_child(&child)
            .try_wait()
            .expect("Unable to wait for child process");
        match waited {
            Some(status) => break status,
            None => sleep(Duration::from_millis(50)),
        }
    };
    drop(raw_mode);
    // Seeing what we came for is the run succeeding, even if the timeout fired
    // in the same instant, so the timeout is only a failure when the run had
    // nothing else to end it.
    if !expectation_met {
        if let Some(timeout) = cli.timeout {
            assert!(
                !timed_out.load(Ordering::SeqCst),
                "Child process did not finish within {timeout} seconds."
            );
        }
    }
    // Being killed by the signal we sent is how a shutdown we asked for looks,
    // not a failure. Without this, pressing Ctrl+C -- the exit path this module
    // goes out of its way to support -- ended every session with a panic. Any
    // other signal is the child dying of something we did not ask for, which is
    // still a failure.
    let shut_down_on_request =
        shutdown_requested.load(Ordering::SeqCst) && status.signal() == Some(SIGKILL);
    assert!(
        status.success() || shut_down_on_request,
        "Child process did not exit successfully. {status}"
    );
    if let Some(expect) = &cli.expect {
        assert!(
            expectation_met,
            "Child process exited without printing {expect:?}."
        );
    }
}

// Asks the child to exit, and records that we were the ones who asked. An error
// from kill means the child has already exited, which is the outcome we wanted.
fn request_shutdown(child: &Mutex<Child>, shutdown_requested: &AtomicBool) {
    shutdown_requested.store(true, Ordering::SeqCst);
    let _ = lock_child(child).kill();
}

fn lock_child(child: &Mutex<Child>) -> std::sync::MutexGuard<'_, Child> {
    child.lock().expect("Child mutex was poisoned")
}

// If child's stdin is piped, this sets the terminal to raw mode and spawns a
// thread that forwards our stdin to child's stdin. The thread shuts the child
// down if Ctrl+C is pressed. Returns a RawTerminal, which reverts the terminal
// to its previous configuration on drop.
fn forward_stdin_if_piped(
    child_stdin: Option<ChildStdin>,
    child: Arc<Mutex<Child>>,
    shutdown_requested: Arc<AtomicBool>,
    ends_on_its_own: bool,
) -> Option<RawTerminal<Stdout>> {
    let mut child_stdin = child_stdin?;
    spawn(move || {
        let our_stdin = stdin();
        let mut our_stdin = our_stdin.lock();
        loop {
            let buffer = our_stdin.fill_buf().expect("Failed to read stdin.");
            if buffer.is_empty() {
                // Our stdin was closed. We interpret this as a signal to exit,
                // because pressing Ctrl+C to trigger an exit is no longer
                // possible. Unless the run already knows how it ends: then a
                // closed stdin just means nobody is typing at us -- the normal
                // case for a script, whose stdin is /dev/null and which would
                // otherwise be cut short before the board had finished booting.
                if ends_on_its_own {
                    return;
                }
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
                // without shutting it down.
                Err(error) if error.kind() == ErrorKind::BrokenPipe => return,

                Err(error) => panic!("Failed to forward stdin: {error}"),
                Ok(bytes) => our_stdin.consume(bytes),
            }
        }
        // Tell the child to exit. Once it does, the main loop will detect the
        // exit and we will shut down cleanly.
        request_shutdown(&child, &shutdown_requested);
    });
    // Raw mode is what lets Ctrl+C reach us rather than the child, but it is
    // only available on a terminal, and it is only any use when the thread
    // above is reading stdin to receive what it delivers. When either end has
    // been redirected -- a CI job, or a `make qemu-example-... </dev/null` --
    // leave the terminal alone. Asking for raw mode without a terminal fails
    // with ENOTTY; asking for it without a stdin to read turns off the
    // terminal's own Ctrl+C while nothing is left to notice the byte it sends
    // instead, leaving no way at all to interrupt the run. Skipping raw mode
    // also skips the CRLF rewriting that only a raw terminal needs.
    if !interactive() {
        return None;
    }
    Some(
        stdout()
            .into_raw_mode()
            .expect("Failed to set terminal to raw mode."),
    )
}

// Spawns a thread that shuts the child down once it elapses, recording that the
// run was cut short rather than ending on its own terms.
fn shut_down_after_timeout(
    child: Arc<Mutex<Child>>,
    shutdown_requested: Arc<AtomicBool>,
    timed_out: Arc<AtomicBool>,
    timeout: u64,
) {
    spawn(move || {
        sleep(Duration::from_secs(timeout));
        timed_out.store(true, Ordering::SeqCst);
        request_shutdown(&child, &shutdown_requested);
        // Killing the child normally ends the run: the main thread sees its
        // stdout close, and reports the timeout. It does not if anything else
        // is holding that pipe open -- a process the child spawned, say -- in
        // which case the main thread stays blocked reading a pipe nothing will
        // ever write to again, and a timeout that cannot end the run is not a
        // backstop. Give the rest of the runner a moment to report this
        // properly, and end the process ourselves if it does not.
        sleep(SHUTDOWN_GRACE);
        eprintln!("Child process did not finish within {timeout} seconds.");
        exit(1);
    });
}

// A string to watch for in the child's output. The child's output arrives in
// whatever sized pieces the pipe hands us, so this keeps the tail of what it
// has seen: without it, a match that straddles two reads would be missed.
struct Expected {
    string: Vec<u8>,
    tail: Vec<u8>,
}

impl Expected {
    fn new(string: &str) -> Self {
        // An empty string has nothing to look for and would make the
        // bookkeeping below underflow. --expect rejects one, so this only has
        // to catch a future caller that does not.
        assert!(!string.is_empty(), "Expected string is empty.");
        Self {
            string: string.into(),
            tail: Vec::new(),
        }
    }

    // Returns true if the string appears in the child's output, once buffer is
    // added to what we have already seen.
    fn seen_in(&mut self, buffer: &[u8]) -> bool {
        self.tail.extend_from_slice(buffer);
        let found = self
            .tail
            .windows(self.string.len())
            .any(|window| window == self.string);
        // Everything before the last string.len() - 1 bytes is too far back to
        // be part of a future match.
        let keep = self.string.len() - 1;
        if self.tail.len() > keep {
            self.tail.drain(..self.tail.len() - keep);
        }
        found
    }
}

// Forwards child's stderr to our stderr if child's stderr is piped, converting
// line endings to CRLF if raw_mode is true.
fn forward_stderr_if_piped(child_stderr: Option<ChildStderr>, raw_mode: bool) {
    let Some(child_stderr) = child_stderr else {
        return;
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

#[cfg(test)]
mod tests {
    use super::Expected;

    #[test]
    fn match_within_one_read() {
        let mut expected = Expected::new("world");
        assert!(!expected.seen_in(b"Entering main loop.\n"));
        assert!(expected.seen_in(b"Hello world!\n"));
    }

    // The child's output arrives in whatever sized pieces the pipe hands us, so
    // a match is not guaranteed to fall inside a single read.
    #[test]
    fn match_split_across_reads() {
        let mut expected = Expected::new("Hello world!");
        assert!(!expected.seen_in(b"He"));
        assert!(!expected.seen_in(b"llo wor"));
        assert!(expected.seen_in(b"ld! tock$ "));
    }

    #[test]
    fn match_longer_than_any_one_read() {
        let mut expected = Expected::new("abcdef");
        assert!(!expected.seen_in(b"ab"));
        assert!(!expected.seen_in(b"cd"));
        assert!(expected.seen_in(b"ef"));
    }

    // A one-byte string has no tail to keep, which is the boundary case of the
    // bookkeeping in seen_in.
    #[test]
    fn single_byte_string() {
        let mut expected = Expected::new("$");
        assert!(!expected.seen_in(b"Hello world!\n"));
        assert!(expected.seen_in(b"tock$ "));
    }

    // A board that never prints what we are waiting for can produce arbitrarily
    // much output, so what we retain of it must stay bounded.
    #[test]
    fn tail_does_not_grow() {
        let mut expected = Expected::new("xyz");
        for _ in 0..1000 {
            assert!(!expected.seen_in(b"Hello world!\n"));
        }
        assert_eq!(expected.tail.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Expected string is empty.")]
    fn empty_string_rejected() {
        Expected::new("");
    }

    // Output the child produced before the string does not become part of it.
    #[test]
    fn no_match_across_a_gap() {
        let mut expected = Expected::new("ab");
        assert!(!expected.seen_in(b"a"));
        assert!(!expected.seen_in(b"cb"));
    }
}
