use libtock_unittest::fake;

type Ac = super::AnalogComparator<fake::Syscalls>;

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let ac = fake::AnalogComparator::new();
    kernel.add_driver(&ac);

    assert_eq!(Ac::exists(), Ok(()));
}

#[test]
fn analog_comparator_comparison() {``
    let kernel = fake::Kernel::new();
    let ac = fake::AnalogComparator::new();
    kernel.add_driver(&ac);

    assert_eq!(Ac::analog_comparator_comparison(1), Ok(()));
}

#[test]
fn analog_comparator_start_comparing() {
    let kernel = fake::Kernel::new();
    let ac = fake::AnalogComparator::new();
    kernel.add_driver(&ac);

    assert_eq!(Ac::analog_comparator_start_comparing(1), Ok(()));
}

#[test]
fn analog_comparator_stop_comparing() {
    let kernel = fake::Kernel::new();
    let ac = fake::AnalogComparator::new();
    kernel.add_driver(&ac);

    assert_eq!(Ac::analog_comparator_stop_comparing(1), Ok(()));
}
