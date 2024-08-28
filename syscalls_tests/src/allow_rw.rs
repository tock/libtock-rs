use libtock_platform::{allow_rw, share, CommandReturn, ErrorCode, Syscalls};
use libtock_unittest::{command_return, fake, DriverInfo, RwAllowBuffer, SyscallLogEntry};
use std::cell::Cell;
use std::rc::Rc;
use std::thread_local;

#[derive(Default)]
struct TestDriver {
    buffer_0: Cell<RwAllowBuffer>,
}

impl fake::SyscallDriver for TestDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(42)
    }

    fn command(&self, _command_num: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        command_return::failure(ErrorCode::NoSupport)
    }

    fn allow_readwrite(
        &self,
        buffer_num: u32,
        buffer: RwAllowBuffer,
    ) -> Result<RwAllowBuffer, (RwAllowBuffer, ErrorCode)> {
        if buffer_num != 0 {
            return Err((buffer, ErrorCode::NoSupport));
        }
        Ok(self.buffer_0.replace(buffer))
    }
}

struct TestConfig;

// CALLED is set to true when returned_nonzero_buffer is called.
thread_local! {static CALLED: Cell<bool> = const {Cell::new(false)} }

impl allow_rw::Config for TestConfig {
    fn returned_nonzero_buffer(driver_num: u32, buffer_num: u32) {
        assert_eq!(driver_num, 42);
        assert_eq!(buffer_num, 0);
        CALLED.with(|cell| cell.set(true));
    }
}

#[test]
fn allow_rw() {
    let kernel = fake::Kernel::new();
    let driver = Rc::new(TestDriver::default());
    kernel.add_driver(&driver);
    let mut buffer1 = [1, 2, 3, 4];
    let mut buffer2 = [5, 6];
    share::scope(|allow_rw| {
        // Tests a call that should fail because it has an incorrect buffer
        // number.
        let result = fake::Syscalls::allow_rw::<TestConfig, 42, 1>(allow_rw, &mut buffer1);
        assert!(!CALLED.with(|c| c.get()));
        assert_eq!(result, Err(ErrorCode::NoSupport));
        assert_eq!(
            kernel.take_syscall_log(),
            [SyscallLogEntry::AllowRw {
                driver_num: 42,
                buffer_num: 1,
                len: 4,
            }]
        );
    });

    // Verify that share::scope unallowed the buffer.
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::AllowRw {
            driver_num: 42,
            buffer_num: 1,
            len: 0,
        }]
    );

    share::scope(|allow_rw| {
        // Tests a call that should succeed and return a zero buffer.
        let result = fake::Syscalls::allow_rw::<TestConfig, 42, 0>(allow_rw, &mut buffer1);
        assert!(!CALLED.with(|c| c.get()));
        assert_eq!(result, Ok(()));
        assert_eq!(
            kernel.take_syscall_log(),
            [SyscallLogEntry::AllowRw {
                driver_num: 42,
                buffer_num: 0,
                len: 4,
            }]
        );

        // Tests a call that should succeed and return a nonzero buffer.
        let result = fake::Syscalls::allow_rw::<TestConfig, 42, 0>(allow_rw, &mut buffer2);
        assert!(CALLED.with(|c| c.get()));
        assert_eq!(result, Ok(()));
        assert_eq!(
            kernel.take_syscall_log(),
            [SyscallLogEntry::AllowRw {
                driver_num: 42,
                buffer_num: 0,
                len: 2,
            }]
        );

        // Mutate the buffer, which under Miri will verify the buffer has been
        // shared with the kernel properly.
        let mut buffer = driver.buffer_0.take();
        buffer[1] = 31;
        driver.buffer_0.set(buffer);
    });

    // Verify that share::scope unallowed the buffer, but only once.
    assert_eq!(
        kernel.take_syscall_log(),
        [SyscallLogEntry::AllowRw {
            driver_num: 42,
            buffer_num: 0,
            len: 0,
        }]
    );

    // Verify the buffer write occurred.
    assert_eq!(buffer2, [5, 31]);
}
