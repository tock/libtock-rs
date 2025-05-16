use crate::fake;
use fake::display::*;
use fake::SyscallDriver;
use libtock_platform::share;
use libtock_platform::CommandReturn;
use libtock_platform::DefaultConfig;
use libtock_platform::Syscalls;
// Tests the command implementation.
#[test]
fn command() {
    let screen = fake::display::Screen::new();

    // Check if the driver exists
    let value = screen.command(EXISTS, 0, 0);
    assert!(CommandReturn::is_success(&value));

    // Verify screen setup command
    assert_eq!(
        screen.command(SCREEN_SETUP, 0, 0).get_success_u32(),
        Some(3)
    );

    // Test power on/off functionality
    assert!(screen.command(SET_POWER, 0, 0).is_success());
    assert_eq!(screen.command(GET_POWER, 0, 0).get_success_u32(), Some(1));

    // Test brightness control
    assert!(screen.command(SET_BRIGHTNESS, 50, 0).is_success());
    assert_eq!(
        screen.command(GET_BRIGHTNESS, 0, 0).get_success_u32(),
        Some(50)
    );

    // Pixel mode and format test
    assert_eq!(
        screen.command(PIXEL_MODES_COUNT, 0, 0).get_success_u32(),
        Some(5)
    );
    assert_eq!(
        screen.command(PIXEL_FORMAT, 0, 0).get_success_u32(),
        Some(332)
    );

    // Set and get rotation
    assert!(screen.command(SET_ROTATION, 90, 0).is_success());
    assert_eq!(
        screen.command(GET_ROTATION, 0, 0).get_success_u32(),
        Some(90)
    );

    // Set and get resolution
    assert!(screen.command(SET_RESOLUTION, 1280, 720).is_success());
    assert_eq!(
        screen.command(GET_RESOLUTION, 0, 0).get_success_2_u32(),
        Some((1280, 720))
    );

    // Test invert mode toggle
    assert!(screen.command(SET_INVERT, 1, 0).is_success());
    assert_eq!(screen.command(GET_INVERT, 0, 0).get_success_u32(), Some(1));

    // Set and get write frame area
    assert!(screen.command(SET_WRITE_FRAME, 360, 720).is_success());
    assert_eq!(
        screen.command(GET_WRITE_FRAME, 0, 0).get_success_2_u32(),
        Some((360, 720))
    );

    // Set and validate pixel format
    assert!(screen.command(SET_PIXEL_FORMAT, 1, 0).is_success());

    // Kernel setup for screen and buffer simulation
    let kernel = fake::Kernel::new();
    kernel.add_driver(&screen);
    let buf = [0; 4];

    // Share buffer with the driver and test WRITE command
    share::scope(|allow_ro| {
        fake::Syscalls::allow_ro::<
            DefaultConfig,
            { fake::display::DRIVER_NUM },
            { fake::display::WRITE_BUFFER_ID },
        >(allow_ro, &buf)
        .unwrap();

        // Execute write operation
        assert!(
            fake::Syscalls::command(fake::display::DRIVER_NUM, fake::display::WRITE, 4, 0)
                .is_success()
        );
    });
}

#[test]
fn kernel_integration() {
    use fake::Kernel;

    let kernel = Kernel::new();
    let screen = Screen::new();
    kernel.add_driver(&screen);

    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 0, 0).is_success());

    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, SCREEN_SETUP, 0, 0).get_success_u32(),
        Some(3)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, SET_POWER, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GET_POWER, 0, 0).get_success_u32(),
        Some(1)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, SET_BRIGHTNESS, 50, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GET_BRIGHTNESS, 0, 0).get_success_u32(),
        Some(50)
    );

    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, PIXEL_MODES_COUNT, 0, 0).get_success_u32(),
        Some(5)
    );

    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, PIXEL_FORMAT, 0, 0).get_success_u32(),
        Some(332)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, SET_ROTATION, 90, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GET_ROTATION, 0, 0).get_success_u32(),
        Some(90)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, SET_RESOLUTION, 1280, 720).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GET_RESOLUTION, 0, 0).get_success_2_u32(),
        Some((1280, 720))
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, SET_INVERT, 1, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GET_INVERT, 0, 0).get_success_u32(),
        Some(1)
    );

    assert!(fake::Syscalls::command(DRIVER_NUM, SET_WRITE_FRAME, 360, 720).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, GET_WRITE_FRAME, 0, 0).get_success_2_u32(),
        Some((360, 720))
    );
}
