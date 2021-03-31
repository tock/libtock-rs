use std::fmt;
use std::io::{BufRead, BufReader};
use std::process::{ChildStdout, Command, Stdio};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tests = Command::new("tock/tools/qemu-build/riscv32-softmmu/qemu-system-riscv32")
        .arg("-M")
        .arg("sifive_e,revb=true")
        .arg("-kernel")
        .arg("tock/target/riscv32imac-unknown-none-elf/release/hifive1")
        .arg("-device")
        .arg("loader,file=target/riscv32imac-unknown-none-elf/tab/hifive1/libtock_test/rv32imac.tbf,addr=0x20040000")
        .arg("-nographic")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = tests.stdout.take().unwrap();
    let child_handle = std::sync::Arc::new(std::sync::Mutex::new(tests));
    let timeout_handle = child_handle.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(10));
        let _ = timeout_handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .kill();
    });
    let result = process_output(stdout);
    let _ = child_handle
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .kill();
    result
}

fn process_output(stdout: ChildStdout) -> Result<(), Box<dyn std::error::Error>> {
    let mut failed_tests = Vec::new();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    for line in stdout_lines {
        let line = line?;
        println!("UART: {}", line);
        let test_result = test_succeeded(line, &mut failed_tests);
        if let Some(true) = test_result {
            return Ok(());
        }
        if let Some(false) = test_result {
            return Err(Box::new(TestError::TestFailure(failed_tests)));
        }
    }
    Err(Box::new(TestError::QemuExit))
}

fn test_succeeded(input: String, failed_tests: &mut Vec<String>) -> Option<bool> {
    let success = input.contains("[      OK ]");
    let failure = input.contains("[ FAILURE ]");
    let input = input.replace("[      OK ]", "");
    let input = input.replace("[ FAILURE ]", "");
    let input = input.trim();
    if input == "Test suite finished with state SUCCESS" && success {
        return Some(true);
    } else if input == "Test suite finished with state FAILURE" && !success {
        return Some(false);
    } else if failure {
        failed_tests.push(input.to_string());
    }
    None
}

#[derive(Debug)]
enum TestError {
    TestFailure(Vec<String>),
    QemuExit,
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestError::TestFailure(failures) => {
                writeln!(f, "A test failure occured. Failed Tests")?;
                for test in failures {
                    writeln!(f, "Test failed\"{}\"", test)?;
                }
                Ok(())
            }
            TestError::QemuExit => write!(f, "Qemu exited unexpectedly."),
        }
    }
}

impl std::error::Error for TestError {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn detects_success_of_test_suite() {
        let mut test_results = Vec::new();
        assert_eq!(
            test_succeeded(
                "[      OK ] Test suite finished with state SUCCESS".into(),
                &mut test_results
            ),
            Some(true)
        );
    }

    #[test]
    pub fn detects_failure_of_test_suite() {
        let mut test_results = Vec::new();
        assert_eq!(
            test_succeeded(
                "[ FAILURE ] Test suite finished with state FAILURE".into(),
                &mut test_results
            ),
            Some(false)
        );
    }

    #[test]
    pub fn detects_test_failures() {
        let mut test_results = Vec::new();
        test_succeeded("[ FAILURE ]  Another test".into(), &mut test_results);
        assert_eq!(test_results, vec!["Another test"]);
    }

    #[test]
    pub fn ignores_other_tests() {
        let mut test_results = Vec::new();
        assert_eq!(
            test_succeeded("[ SUCCESS ]  Another test".into(), &mut test_results),
            None
        );
    }
}
