use libtock_unittest::fake;

type LedsFactory = super::LedsFactory<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    let mut leds_factory = LedsFactory::new();
    assert!(leds_factory.init_driver().is_err());
}

#[test]
fn driver_check() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    let mut leds_factory = LedsFactory::new();
    let leds_driver = leds_factory.init_driver().unwrap();

    for led in 0..10 {
        assert!(leds_driver.get(led).is_ok());
    }
}

#[test]
fn num_leds() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    let mut leds_factory = LedsFactory::new();
    let leds_driver = leds_factory.init_driver().unwrap();

    assert_eq!(leds_driver.num_leds(), 10);
}

#[test]
fn on() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    let mut leds_factory = LedsFactory::new();
    let leds_driver = leds_factory.init_driver().unwrap();

    leds_driver.get(0).unwrap().on().unwrap();
    assert_eq!(driver.get_led(0), Some(true));
}

#[test]
fn off() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    let mut leds_factory = LedsFactory::new();
    let leds_driver = leds_factory.init_driver().unwrap();

    leds_driver.get(0).unwrap().off().unwrap();

    assert_eq!(driver.get_led(0), Some(false));
}

#[test]
fn toggle() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    let mut leds_factory = LedsFactory::new();
    let leds_driver = leds_factory.init_driver().unwrap();

    leds_driver.get(0).unwrap().toggle().unwrap();
    assert_eq!(driver.get_led(0), Some(true));
    leds_driver.get(0).unwrap().toggle().unwrap();
    assert_eq!(driver.get_led(0), Some(false));
}

#[test]
fn on_off() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    let mut leds_factory = LedsFactory::new();
    let leds_driver = leds_factory.init_driver().unwrap();

    leds_driver.get(0).unwrap().on().unwrap();
    assert_eq!(driver.get_led(0), Some(true));
    leds_driver.get(0).unwrap().off().unwrap();
    assert_eq!(driver.get_led(0), Some(false));
}

#[test]
fn no_led() {
    let kernel = fake::Kernel::new();
    let driver = fake::Leds::<10>::new();
    kernel.add_driver(&driver);

    let mut leds_factory = LedsFactory::new();
    let leds_driver = leds_factory.init_driver().unwrap();

    assert!(leds_driver.get(11).is_err());
}
