use libtock_platform::{RawSyscalls, Register};
use libtock_unittest::fake::{self, ieee802154::Frame as FakeFrame, Ieee802154Phy};

/// The Ieee8021514Phy userspace driver calls yield_wait() immediately after subscribe().
/// Normally, it would wait for the kernel to receive a frame and then asynchronously
/// schedule an upcall, but in this testing framework it is required to schedule
/// an upcall before yield_wait(), because otherwise a panic is raised.
///
/// HACK: This wraps around fake::Syscalls to hook subscribe::FRAME_RECEIVED
/// so that immediately after subscribing for the upcall, frames are received
/// by the kernel driver and the corresponding upcall is scheduled.
struct FakeSyscalls;

unsafe impl RawSyscalls for FakeSyscalls {
    unsafe fn yield1([r0]: [Register; 1]) {
        libtock_unittest::fake::Syscalls::yield1([r0])
    }

    unsafe fn yield2([r0, r1]: [Register; 2]) {
        libtock_unittest::fake::Syscalls::yield2([r0, r1])
    }

    unsafe fn syscall1<const CLASS: usize>([r0]: [Register; 1]) -> [Register; 2] {
        libtock_unittest::fake::Syscalls::syscall1::<CLASS>([r0])
    }

    unsafe fn syscall2<const CLASS: usize>([r0, r1]: [Register; 2]) -> [Register; 2] {
        libtock_unittest::fake::Syscalls::syscall2::<CLASS>([r0, r1])
    }

    unsafe fn syscall4<const CLASS: usize>([r0, r1, r2, r3]: [Register; 4]) -> [Register; 4] {
        let trigger_rx_upcall = match CLASS {
            libtock_platform::syscall_class::SUBSCRIBE => {
                let driver_num: u32 = r0.try_into().unwrap();
                let subscribe_num: u32 = r1.try_into().unwrap();
                let len: usize = r3.into();
                assert_eq!(driver_num, DRIVER_NUM);

                subscribe_num == subscribe::FRAME_RECEIVED && len > 0
            }
            _ => false,
        };

        let ret = libtock_unittest::fake::Syscalls::syscall4::<CLASS>([r0, r1, r2, r3]);
        if trigger_rx_upcall {
            if let Some(driver) = Ieee802154Phy::instance() {
                driver.driver_receive_pending_frames();

                if driver.has_pending_rx_frames() {
                    driver.trigger_rx_upcall();
                }
            }
        }
        ret
    }
}

use crate::{subscribe, DRIVER_NUM};

use super::{RxOperator, RxRingBuffer};

type Ieee802154 = super::Ieee802154<FakeSyscalls>;
type RxSingleBufferOperator<'buf, const N: usize> =
    super::RxSingleBufferOperator<'buf, N, FakeSyscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert!(!Ieee802154::exists());
}

#[test]
fn exists() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ieee802154Phy::new();
    kernel.add_driver(&driver);

    assert!(Ieee802154::exists());
}

#[test]
fn configure() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ieee802154Phy::new();
    kernel.add_driver(&driver);

    let pan: u16 = 0xcafe;
    let addr_short: u16 = 0xdead;
    let addr_long: u64 = 0xdeaddad;
    let tx_power: i8 = -0x42;
    let channel: u8 = 0xff;

    Ieee802154::set_pan(pan);
    Ieee802154::set_address_short(addr_short);
    Ieee802154::set_address_long(addr_long);
    Ieee802154::set_tx_power(tx_power).unwrap();
    Ieee802154::set_channel(channel).unwrap();

    Ieee802154::commit_config();

    assert_eq!(Ieee802154::get_pan().unwrap(), pan);
    assert_eq!(Ieee802154::get_address_short().unwrap(), addr_short);
    assert_eq!(Ieee802154::get_address_long().unwrap(), addr_long);
    assert_eq!(Ieee802154::get_channel().unwrap(), channel);
    assert_eq!(Ieee802154::get_tx_power().unwrap(), tx_power);
}

#[test]
fn transmit_frame() {
    let kernel = fake::Kernel::new();
    let driver = fake::Ieee802154Phy::new();
    kernel.add_driver(&driver);

    Ieee802154::transmit_frame(b"foo").unwrap();
    Ieee802154::transmit_frame(b"bar").unwrap();
    assert_eq!(
        driver.take_transmitted_frames(),
        &[&b"foo"[..], &b"bar"[..]],
    );
}

mod rx {
    use super::*;
    fn test_with_driver(test: impl FnOnce(&Ieee802154Phy)) {
        let kernel = fake::Kernel::new();
        let driver = fake::Ieee802154Phy::new();
        kernel.add_driver(&driver);

        test(&driver)
    }

    fn test_with_single_buf_operator<const SUPPORTED_FRAMES: usize>(
        driver: &Ieee802154Phy,
        test: impl Fn(&Ieee802154Phy, &mut dyn RxOperator),
    ) {
        let mut buf = RxRingBuffer::<SUPPORTED_FRAMES>::new();
        let mut operator = RxSingleBufferOperator::new(&mut buf);

        test(driver, &mut operator)
    }

    fn no_frame_comes(_driver: &Ieee802154Phy, operator: &mut dyn RxOperator) {
        // No frame is available, so we expect to panic in tests,
        // because yield_wait is called without pending upcalls.
        // THIS PANICS
        let _ = operator.receive_frame();
    }

    #[test]
    #[should_panic = "yield-wait called with no queued upcall"]
    fn no_frame_comes_single_buf() {
        test_with_driver(|driver| {
            const SUPPORTED_FRAMES: usize = 2;

            test_with_single_buf_operator::<SUPPORTED_FRAMES>(driver, no_frame_comes);
        });
    }

    #[test]
    fn receive_frame() {
        test_with_driver(|driver| {
            const SUPPORTED_FRAMES: usize = 2;

            test_with_single_buf_operator::<SUPPORTED_FRAMES>(driver, |driver, operator| {
                let frame1 = b"alamakota";

                driver.radio_receive_frame(FakeFrame::with_body(frame1));
                // Now one frame is available.

                let got_frame1 = operator.receive_frame().unwrap();
                assert_eq!(got_frame1.payload_len as usize, frame1.len());
                assert_eq!(
                    &got_frame1.body[..got_frame1.payload_len as usize],
                    &frame1[..]
                );
            });
        });
    }

    fn only_one_frame_comes(driver: &Ieee802154Phy, operator: &mut dyn RxOperator) {
        let frame1 = b"alamakota";

        // Now one frame is available.
        driver.radio_receive_frame(FakeFrame::with_body(frame1));
        let got_frame1 = operator.receive_frame().unwrap();
        assert_eq!(got_frame1.payload_len as usize, frame1.len());
        assert_eq!(&got_frame1.body[..frame1.len()], frame1);

        // But only one!
        // THIS PANICS
        let _ = operator.receive_frame();
    }

    #[test]
    #[should_panic = "yield-wait called with no queued upcall"]
    fn receive_frame_only_one_single_buf() {
        test_with_driver(|driver| {
            const SUPPORTED_FRAMES: usize = 2;

            test_with_single_buf_operator::<SUPPORTED_FRAMES>(driver, only_one_frame_comes);
        });
    }

    #[test]
    fn receive_many_frames() {
        test_with_driver(|driver| {
            const SUPPORTED_FRAMES: usize = 3;

            test_with_single_buf_operator::<{ SUPPORTED_FRAMES + 1 }>(
                driver,
                |driver, operator| {
                    for (times, frame) in
                        [1, 2, 3, 10]
                            .iter()
                            .copied()
                            .zip([&b"one"[..], b"two", b"three", b"ten"])
                    {
                        for _ in 0..times {
                            driver.radio_receive_frame(FakeFrame::with_body(frame));
                        }

                        for _ in 0..core::cmp::min(times, SUPPORTED_FRAMES) {
                            let got_frame = operator.receive_frame().unwrap();
                            let expected_frame = frame;
                            assert_eq!(got_frame.payload_len as usize, expected_frame.len());
                            assert_eq!(
                                &got_frame.body[..got_frame.payload_len as usize],
                                expected_frame
                            );
                        }
                    }
                },
            );
        });
    }

    #[test]
    fn receive_various_frames() {
        test_with_driver(|driver| {
            const SUPPORTED_FRAMES: usize = 3;

            test_with_single_buf_operator::<{ SUPPORTED_FRAMES + 1 }>(
                driver,
                |driver, operator| {
                    let frame1 = b"alamakota";
                    let frame2 = b"ewamamewe";
                    let frame3 = b"wojciechmalaptop";
                    let frames: [&[u8]; 3] = [frame1, frame2, frame3];

                    let order = [0, 1, 2, 2, 1, 0, 2, 2, 1, 0, 2];
                    for idx in order {
                        let times = idx + 1;

                        for _ in 0..times {
                            driver.radio_receive_frame(FakeFrame::with_body(frames[idx]));
                        }

                        for _ in 0..core::cmp::min(times, SUPPORTED_FRAMES) {
                            let got_frame = operator.receive_frame().unwrap();
                            let expected_frame = frames[idx];
                            assert_eq!(got_frame.payload_len as usize, expected_frame.len());
                            assert_eq!(
                                &got_frame.body[..got_frame.payload_len as usize],
                                expected_frame
                            );
                        }
                    }
                },
            );
        });
    }
}
