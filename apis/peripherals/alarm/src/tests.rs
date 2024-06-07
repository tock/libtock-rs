use libtock_unittest::fake;

use crate::{Hz, Milliseconds, Ticks};

type Alarm = crate::Alarm<fake::Syscalls>;

#[test]
fn get_freq() {
    let kernel = fake::Kernel::new();
    let driver = fake::Alarm::new(1000);
    kernel.add_driver(&driver);
    assert_eq!(Alarm::get_frequency(), Ok(Hz(1000)));
}

#[test]
fn sleep() {
    let kernel = fake::Kernel::new();
    let driver = fake::Alarm::new(1000);
    kernel.add_driver(&driver);

    assert_eq!(Alarm::sleep_for(Ticks(0)), Ok(()));
    assert_eq!(Alarm::sleep_for(Ticks(1000)), Ok(()));
    assert_eq!(Alarm::sleep_for(Milliseconds(1000)), Ok(()));
}
