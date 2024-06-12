use crate::fake::{self, SyscallDriver};
use fake::analog_comparator::*;
use libtock_platform::ErrorCode;

#[test]
fn command() {
    let analog_comparator = AnalogComparator::new();
    assert!(analog_comparator.command(EXISTS, 1, 2).is_success());

    assert!(analog_comparator.command(1, 0, 0).is_success());

    assert_eq!(
        analog_comparator.command(1, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    analog_comparator.set_value(100);
    assert!(analog_comparator.command(1, 0, 1).is_success());
}

#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let analog_comparator = AnalogComparator::new();
    kernel.add_driver(&analog_comparator);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, 1, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, 1, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    analog_comparator.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, 1, 0, 1).is_success());
}
