use libtock_platform::ErrorCode;
use libtock_unittest::{fake, ExpectedSyscall};

type Screen = super::Display<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Screen::exists(), Err(ErrorCode::Fail))
}
#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::exists(), Ok(()));
}

#[test]
fn screen_setup() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::screen_setup(), Ok(3));
}

#[test]
fn set_power() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::set_power(1), Ok(()));
}

#[test]
fn set_brightness() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_brightness(90), Ok(()));
}

#[test]
fn set_invert_on() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_invert_on(), Ok(()));
}

#[test]
fn set_invert_off() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_invert_off(), Ok(()));
}

#[test]
fn set_invert() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::set_invert(2), Ok(()));
}

#[test]
fn get_resolution_modes_count() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::get_resolution_modes_count(), Ok(2));
}

#[test]
fn get_resolution_width_height() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::get_resolution_width_height(0), Ok((1920, 1080)));
    assert_eq!(Screen::get_resolution_width_height(1), Ok((2560, 1440)));
    assert_eq!(Screen::get_resolution_width_height(2), Ok((1280, 720)));
}
#[test]
fn get_resolution_width_height_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(
        Screen::get_resolution_width_height(3),
        Err(ErrorCode::Invalid)
    );
}
#[test]
fn pixel_modes_count() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::pixel_modes_count(), Ok(5));
}

#[test]
fn get_screen_pixel_format_success() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::pixel_format(0), Ok(332));
    assert_eq!(Screen::pixel_format(1), Ok(565));
}

#[test]
fn get_screen_pixel_format_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::pixel_format(8), Err(ErrorCode::Invalid));
}

#[test]
fn set_rotation_success() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_rotation(30), Ok(()));
}

#[test]
fn set_rotation_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::set_rotation(360), Err(ErrorCode::Invalid));
}

#[test]
fn get_rotation_success() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_rotation(30), Ok(()));
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::get_rotation(), Ok(30));
}

#[test]
fn get_rotation_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::get_rotation(), Ok(0));
}

#[test]
fn set_resolution() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_resolution(360, 720), Ok(()));
}

#[test]
fn get_resolution_sucess() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_resolution(360, 720), Ok(()));
    assert_eq!(Screen::get_resolution(), Ok((360, 720)));
}

#[test]
fn get_resolution_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::get_resolution(), Ok((0, 0)));
}

#[test]
fn set_pixel_format() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_pixel_format(2), Ok(()));
}

#[test]
fn get_pixel_format_success() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });
    assert_eq!(Screen::set_pixel_format(2), Ok(()));
    assert_eq!(Screen::get_pixel_format(), Ok(2));
}

#[test]
fn get_pixel_format_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::get_pixel_format(), Ok(0));
}

#[test]
fn set_write_frame() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    assert_eq!(Screen::set_write_frame(10, 20, 30, 30), Ok(()));
}

#[test]
fn write_buffer() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    let _ = Screen::set_pixel_format(2);
    let buffer = [0u8; 4];

    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: 0x90001,
        buffer_num: 0,
        return_error: None,
    });

    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });

    assert_eq!(Screen::write(&buffer), Ok(()));
}

#[test]
fn fill_success() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    let _ = Screen::set_pixel_format(2);
    let mut buffer = [0u8; 2];
    let color = 0xF800;
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: 0x90001,
        buffer_num: 0,
        return_error: None,
    });

    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });

    assert_eq!(Screen::fill(&mut buffer, color), Ok(()));
}

#[test]
fn fill_fail() {
    let kernel = fake::Kernel::new();
    let driver = fake::Screen::new();
    kernel.add_driver(&driver);
    let _ = Screen::set_pixel_format(2);
    let mut buffer = [0u8; 1];
    let color = 0xF800;
    kernel.add_expected_syscall(ExpectedSyscall::AllowRo {
        driver_num: 0x90001,
        buffer_num: 0,
        return_error: None,
    });

    kernel.add_expected_syscall(ExpectedSyscall::Subscribe {
        driver_num: 0x90001,
        subscribe_num: 0,
        skip_with_error: None,
    });

    assert_eq!(Screen::fill(&mut buffer, color), Err(ErrorCode::Fail));
}
