use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, StructOpt)]
#[structopt(name = "libtock test runner", about = "run libtock tests")]
struct Arguments {
    #[structopt(short, long)]
    timeout_ms: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    timeout(Duration::from_secs(10), perform_tests()).await?
}

async fn perform_tests() -> Result<(), Box<dyn std::error::Error>> {
    let  tests = Command::new("qemu-system-riscv32")
        .arg("-M")
        .arg("sifive_e")
        .arg("-kernel")
        .arg("../../tock/boards/hifive1/target/riscv32imac-unknown-none-elf/release/hifive1")
        .arg("-device")
        .arg("loader,file=./../target/riscv32imac-unknown-none-elf/tab/hifive1/libtock_test/rv32imac.tbf,addr=0x20430000")
        .arg("-nographic")
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = tests.stdout.unwrap();

    let stdout_reader = BufReader::new(stdout);
    let mut stdout_lines = stdout_reader.lines();
    let mut result_map = HashMap::<TockTest, bool>::new();

    while let Some(line) = stdout_lines.next_line().await? {
        println!("UART: {}", line);
        if let Some(entry) = parse_line(line) {
            result_map.insert(entry.0, entry.1);
        }
        if result_map.keys().len() >= NUMBER_OF_TESTS {
            if result_map.values().all(|x| *x) {
                return Ok(());
            } else {
                panic!();
            }
        }
    }
    panic!("qemu exited unexpectedly")
}

fn parse_line(input: String) -> Option<(TockTest, bool)> {
    let result = input.find("[      OK ]").is_some();
    let input = input.replace("[      OK ]", "");
    let input = input.replace("[ FAILURE ]", "");
    let input = input.trim();
    let tock_test = label_to_tocktest(input.to_string());
    tock_test.map(|test| (test, result))
}

const NUMBER_OF_TESTS: usize = 6;
#[derive(PartialEq, Debug, Hash, Eq)]
enum TockTest {
    Console,
    StaticMut,
    DynamicDispatch,
    Formatting,
    Heap,
    DriversOnlyInstantiableOnce,
}

fn label_to_tocktest(label: String) -> Option<TockTest> {
    match label.as_str() {
        "Console" => Some(TockTest::Console),
        "static mut" => Some(TockTest::StaticMut),
        "Dynamic dispatch" => Some(TockTest::DynamicDispatch),
        "Formatting" => Some(TockTest::Formatting),
        "Heap" => Some(TockTest::Heap),
        "Drivers only instantiable once" => Some(TockTest::DriversOnlyInstantiableOnce),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn works() {
        assert_eq!(
            parse_line("[      OK ] Console".into()),
            Some((TockTest::Console, true))
        );
    }

    #[test]
    pub fn fails() {
        assert_eq!(parse_line("[      OK ]  No Console".into()), None);
    }
}
