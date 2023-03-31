use crate::fake;
use fake::buzzer::*;
use libtock_platform::ErrorCode;

#[test]
fn command() {
    use fake::SyscallDriver;
    let buzzer = Buzzer::new();
    let value = buzzer.command(DRIVER_CHECK, 1, 2);
    assert_eq!(value.get_success_u32(), Some(1));
    assert_eq!(
        buzzer.command(BUZZER_ON, 0, 0).get_failure(),
        Some(ErrorCode::Invalid)
    );
    assert_eq!(buzzer.get_buzzer(), false);
    assert!(buzzer.command(BUZZER_ON, 1, 1).is_success());
    assert_eq!(buzzer.get_buzzer(), true);
    assert!(buzzer.command(BUZZER_OFF, 0, 0).is_success());
    assert_eq!(buzzer.get_buzzer(), false);
}
