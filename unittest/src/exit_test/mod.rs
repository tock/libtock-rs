//! Tools for testing code that calls the Exit system call.
//!
//! This module is not compatible with Miri because it requires the ability to
//! spawn external processes, which Miri does not support by default. Therefore
//! it is only available for non-Miri tests.

#[cfg(test)]
mod tests;

use std::panic::{catch_unwind, Location, UnwindSafe};

/// Utility for testing code that is expected to call the Exit system call. It
/// is used as follows (inside a unit test case):
///
/// ```
/// // Note: exit_test is not available in Miri
/// #[cfg(miri)]
/// fn main() {}
///
/// #[cfg(not(miri))]
/// fn main() {
///     use libtock_platform::Syscalls;
///     let _kernel = libtock_unittest::fake::Kernel::new();
///     let exit = libtock_unittest::exit_test("tests::foo", || {
///         libtock_unittest::fake::Syscalls::exit_terminate(0);
///     });
///     assert_eq!(exit, libtock_unittest::ExitCall::Terminate(0));
/// }
/// ```
///
/// `exit_test` will panic (to fail the test case) if the code does not call
/// Exit, or if the parameters to exit do not match `expected_exit`.
///
/// `test_name` must match the name of the test case, as is used in Rust's test
/// framework's filter syntax.
///
/// `exit_test` is a hack, and the user should understand how it works to
/// understand its limitations. When the above test case is executed, the
/// following happens:
///
/// 1. The first test process (the one started by the user, e.g. through
///    `cargo test`) executes the `foo()` test case, which calls `exit_test`.
///    We'll call this process A, as it was the first test process to start.
/// 2. `exit_test` spawns a second process, B, by invoking the same test binary
///    as process A. When it does, it passes a filter to process B telling it
///    to only invoke `foo()` (this is the purpose of the `test_name` argument).
///    It also sets an environment variable telling process B that `exit_test`
///    launched it.
/// 3. Process B runs the `foo()` test case, which invokes `exit_test` a second
///    time.
/// 4. `exit_test` in process B uses the environment variable to detect that it
///    is the subprocess version, and it runs closure `fcn`. If `fcn` does not
///    call Exit, it panics. `exit_test` will not return from process B.
/// 5. `exit_test` in process A waits until process B terminates.
/// 6. `exit_test` in process A reads the output of process B to determine
///    whether Exit was called, and if so what arguments were called.
/// 7. `exit_test` in process A returns a value indicating what happened in
///    process B, which `foo()` can make assertions on.
#[track_caller]
pub fn exit_test<F: FnOnce() + UnwindSafe>(test_name: &str, fcn: F) -> ExitCall {
    if let Some(signal_var) = std::env::var_os(SIGNAL_VAR) {
        // We are process B, run the test function.
        run_test(signal_var, fcn)
    } else {
        // We are process A, spawn process B.
        spawn_test(test_name)
    }
}

/// Indicates what type of Exit call was performed, and what completion code was
/// provided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitCall {
    Terminate(u32),
    Restart(u32),
}

// -----------------------------------------------------------------------------
// Public API above, implementation details below.
// -----------------------------------------------------------------------------

// Prints a message telling exit_test the Exit system call was called.
pub(crate) fn signal_exit(exit_call: ExitCall) {
    signal_message(ExitMessage::ExitCall(exit_call));
}

#[doc(hidden)]
impl std::fmt::Display for ExitCall {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ExitCall::Terminate(code) => write!(f, "exit-terminate({})", code),
            ExitCall::Restart(code) => write!(f, "exit-restart({})", code),
        }
    }
}

#[doc(hidden)]
impl std::str::FromStr for ExitCall {
    type Err = ParseExitError;

    fn from_str(s: &str) -> Result<ExitCall, ParseExitError> {
        // Strip off the trailing ), leaving the name and (
        let s = s.strip_suffix(')').ok_or(ParseExitError)?;

        if let Some(s) = s.strip_prefix("exit-terminate(") {
            Ok(ExitCall::Terminate(s.parse().or(Err(ParseExitError))?))
        } else if let Some(s) = s.strip_prefix("exit-restart(") {
            Ok(ExitCall::Restart(s.parse().or(Err(ParseExitError))?))
        } else {
            Err(ParseExitError)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[doc(hidden)]
pub struct ParseExitError;

// The name of the environment variable used by process A to tell process B that
// it is process B. The value of the environment variable is the location where
// exit_test was called (this location is used to help verify that test_name is
// correct).
const SIGNAL_VAR: &str = "LIBTOCK_UNITTEST_EXIT_TEST";

// This string is printed by process B to tell process A how it exited. It is
// followed by the Display string for a ExitMessage.
const EXIT_STRING: &str = "LIBTOCK_UNITTEST_EXIT_TEST_RESULT: ";

#[derive(Debug, Eq, PartialEq)]
enum ExitMessage {
    ExitCall(ExitCall),
    WrongCase,
    DidNotExit,
}

impl std::fmt::Display for ExitMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ExitMessage::ExitCall(exit_call) => write!(f, "ExitCall({})", exit_call),
            ExitMessage::WrongCase => write!(f, "WrongCase"),
            ExitMessage::DidNotExit => write!(f, "DidNotExit"),
        }
    }
}

impl std::str::FromStr for ExitMessage {
    type Err = ParseExitError;

    fn from_str(s: &str) -> Result<ExitMessage, ParseExitError> {
        if let Some(s) = s.strip_prefix("ExitCall(") {
            let s = s.strip_suffix(')').ok_or(ParseExitError)?;
            Ok(ExitMessage::ExitCall(s.parse()?))
        } else if s == "WrongCase" {
            Ok(ExitMessage::WrongCase)
        } else if s == "DidNotExit" {
            Ok(ExitMessage::DidNotExit)
        } else {
            Err(ParseExitError)
        }
    }
}

// Implements process A's behavior for exit_test: spawns this test again as a
// subprocess, only executing the test specified by test_name.
#[track_caller]
fn spawn_test(test_name: &str) -> ExitCall {
    let current_exe = std::env::current_exe().expect("Unable to find test executable");
    let output = std::process::Command::new(current_exe)
        .args(std::env::args_os())
        .arg("--nocapture")
        .arg("--exact")
        .arg(test_name)
        .envs(std::env::vars_os())
        .env(SIGNAL_VAR, format!("{}", Location::caller()))
        .output()
        .expect("Subprocess exec failed");
    let stdout = String::from_utf8(output.stdout).expect("Subprocess produced invalid UTF-8");
    println!("{} subprocess stdout:\n{}", test_name, stdout);
    let stderr = String::from_utf8(output.stderr).expect("Subprocess produced invalid UTF-8");
    println!("{} subprocess stderr:\n{}", test_name, stderr);

    // Search for the exit message in stdout.
    for line in stdout.lines() {
        if let Some(message) = line.strip_prefix(EXIT_STRING) {
            match message
                .parse::<ExitMessage>()
                .expect("Failed to parse exit message")
            {
                ExitMessage::ExitCall(exit_call) => return exit_call,
                ExitMessage::WrongCase => panic!(
                    "Subprocess executed the wrong test case. Perhaps test_name is incorrect?"
                ),
                ExitMessage::DidNotExit => panic!("Subprocess did not call Exit."),
            }
        }
    }
    panic!("Subprocess did not indicate why it exited. Perhaps test_name is incorrect?");
}

// Used by process B to send a message to process A.
fn signal_message(message: ExitMessage) {
    println!("{}{}", EXIT_STRING, message);
}

// Implements process B's behavior for exit_test. Verifies the test case was
// specified correctly, runs the test function, and prints an error if the test
// function did not call Exit.
#[track_caller]
fn run_test<F: FnOnce() + UnwindSafe>(signal_var: std::ffi::OsString, fcn: F) -> ! {
    let signal_var = signal_var.to_str().expect("Invalid signal variable value");
    if format!("{}", Location::caller()) != signal_var {
        signal_message(ExitMessage::WrongCase);
        std::process::exit(1);
    }
    println!("exit_test: closure return value {:?}", catch_unwind(fcn));
    signal_message(ExitMessage::DidNotExit);
    std::process::exit(1);
}
