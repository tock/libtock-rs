use crate::fake;
use fake::alarm::*;

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;
    let alarm = Alarm::new(10);

    assert_eq!(
        alarm.command(command::FREQUENCY, 1, 2).get_success_u32(),
        Some(10)
    );
}
