use libtock_platform::ErrorCode;
use libtock_unittest::fake;

type UartController = super::UartController<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(UartController::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn write_and_read() {
    let kernel = fake::Kernel::new();
    let driver = fake::UartController::new();
    kernel.add_driver(&driver);

    let tx = [1u8, 2, 3];
    assert_eq!(
        UartController::uart_controller_write_sync(0, &tx, tx.len() as u32),
        Ok(())
    );
    assert_eq!(driver.get_last_write(), tx);

    driver.set_read_data(&[9, 8, 7]);
    let mut rx = [0u8; 3];
    assert_eq!(
        UartController::uart_controller_read_sync(1, &mut rx, 3),
        Ok(())
    );
    assert_eq!(rx, [9, 8, 7]);

    assert_eq!(
        UartController::uart_controller_write_sync(2, &tx, 3),
        Err(ErrorCode::Invalid)
    );
}
