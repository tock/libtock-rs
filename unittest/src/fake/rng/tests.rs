use libtock_platform::{share, DefaultConfig};

use crate::{fake, RwAllowBuffer};

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;

    let rng = fake::Rng::new();

    // Test driver response to an existence check request
    assert!(rng.command(fake::rng::EXISTS, 0, 0).is_success());

    // Test driver response for sharing a buffer
    // to a valid and invalid buffer index, respectively
    assert!(rng.allow_readwrite(0, RwAllowBuffer::default()).is_ok());
    assert!(rng.allow_readwrite(1, RwAllowBuffer::default()).is_err());
}

// Integration test that verifies Console works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;

    let kernel = fake::Kernel::new();
    let rng = fake::Rng::new();
    // Predetermined sample of RNG data "read" from the RNG low-level driver
    // that will be put into the shared buffer
    let bytes = [0_u8, 1_u8, 2_u8, 3_u8, 4_u8];
    rng.bytes.set(bytes.to_vec());
    kernel.add_driver(&rng);

    // Test driver response to an existence check request
    assert!(fake::Syscalls::command(fake::rng::DRIVER_NUM, 0, 0, 0).is_success());

    // Buffer to be shared with the kernel and filled with random bytes
    let mut buffer: [u8; 5] = [0; 5];
    share::scope(|allow_rw| {
        // Register the provided buffer
        fake::Syscalls::allow_rw::<DefaultConfig, { fake::rng::DRIVER_NUM }, 0>(
            allow_rw,
            &mut buffer,
        )
        .unwrap();
        // Test driver response to buffer fill requests
        assert!(fake::Syscalls::command(fake::rng::DRIVER_NUM, 1, 5, 0).is_success());
    });
    // Additionally check that the buffer has been correctly filled
    assert_eq!(buffer.cmp(&bytes), core::cmp::Ordering::Equal);
}
