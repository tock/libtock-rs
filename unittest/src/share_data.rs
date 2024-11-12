use crate::kernel_data::with_kernel_data;
use crate::upcall::{UpcallId, UpcallQueueEntry};
use std::cell::Cell;

/// A reference used by a `fake::SyscallDriver` to access data shared between it
/// and the `fake::Kernel`, such as upcalls.
///
/// A `Default`-initialized `DriverShareRef` links to an empty share. It can be
/// used as normal, but is generally a no-op. This allows `fake::SyscallDriver`
/// implementations to store a `DriverShareRef` directly, rather than having to
/// contain a `Cell<Option<DriverShareRef>>`.
#[derive(Default)]
pub struct DriverShareRef {
    pub(crate) driver_num: Cell<u32>,
}

impl DriverShareRef {
    /// Replaces this DriverShareRef with another. `fake:SyscallDrivers` can use
    /// `replace` to implement `register` to avoid having to store their
    /// `DriverShareRef` inside a `Cell`.
    pub fn replace(&self, new: Self) {
        self.driver_num.set(new.driver_num.get());
    }

    /// Schedules the upcall with the specified subscribe number. Like the real
    /// kernel, this does nothing if there is no upcall with number
    /// `subscribe_num` or the upcall is the null upcall.
    pub fn schedule_upcall(
        &self,
        subscribe_num: u32,
        args: (u32, u32, u32),
    ) -> Result<(), InvalidSubscribeNum> {
        with_kernel_data(|kernel_data| {
            let kernel_data = match kernel_data {
                Some(kernel_data) => kernel_data,
                None => return Ok(()),
            };
            let driver_data = kernel_data
                .drivers
                .get(&self.driver_num.get())
                .expect("DriverShareRef: registered but nonexistent?");
            if subscribe_num >= driver_data.num_upcalls {
                return Err(InvalidSubscribeNum {
                    upcall_count: driver_data.num_upcalls,
                    requested: subscribe_num,
                });
            }
            let upcall = match driver_data.upcalls.get(&subscribe_num) {
                Some(&upcall) => upcall,
                None => return Ok(()),
            };
            // Don't bother queueing a null upcall, as they don't do anything
            // when invoked anyway, and the core kernel does not queue them
            // either.
            if upcall.is_null() {
                return Ok(());
            }
            kernel_data.upcall_queue.push_back(UpcallQueueEntry {
                args,
                id: UpcallId {
                    driver_num: self.driver_num.get(),
                    subscribe_num,
                },
                upcall,
            });
            Ok(())
        })
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[error("Upcall number {requested} too large, expected < {upcall_count}.")]
pub struct InvalidSubscribeNum {
    requested: u32,
    upcall_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upcall::Upcall;
    use crate::DriverInfo;
    use libtock_platform::Register;
    use std::rc::Rc;

    #[derive(Default)]
    struct MockDriver {
        share_ref: DriverShareRef,
    }
    impl crate::fake::SyscallDriver for MockDriver {
        fn info(&self) -> DriverInfo {
            DriverInfo::new(1).upcall_count(10)
        }
        fn register(&self, share_ref: DriverShareRef) {
            self.share_ref.replace(share_ref);
        }
        fn command(&self, _: u32, _: u32, _: u32) -> libtock_platform::CommandReturn {
            crate::command_return::failure(libtock_platform::ErrorCode::NoSupport)
        }
    }

    #[test]
    fn schedule_errors() {
        let mock_driver = Rc::new(MockDriver::default());
        let kernel = crate::fake::Kernel::new();
        kernel.add_driver(&mock_driver);
        assert_eq!(
            mock_driver.share_ref.schedule_upcall(10, (3, 4, 5)),
            Err(InvalidSubscribeNum {
                upcall_count: 10,
                requested: 10
            })
        );
    }

    #[test]
    fn schedule_success() {
        let mock_driver = Rc::new(MockDriver::default());
        let kernel = crate::fake::Kernel::new();
        kernel.add_driver(&mock_driver);

        // Call schedule with no registered upcall.
        assert_eq!(mock_driver.share_ref.schedule_upcall(2, (3, 4, 5)), Ok(()));
        with_kernel_data(|kernel_data| {
            let kernel_data = kernel_data.unwrap();

            // There was no upcall to schedule, so the queue should still be
            // empty.
            assert!(kernel_data.upcall_queue.is_empty());

            // Register a null upcall.
            kernel_data.drivers.get_mut(&1).unwrap().upcalls.insert(
                2,
                Upcall {
                    fn_pointer: None,
                    data: 1234u32.into(),
                },
            );
        });
        // Call schedule again. This should still do nothing, because the upcall
        // is a null upcall.
        assert_eq!(mock_driver.share_ref.schedule_upcall(2, (3, 4, 5)), Ok(()));
        unsafe extern "C" fn upcall(_: u32, _: u32, _: u32, _: libtock_platform::Register) {}
        // Cast to a pointer to get a stable address.
        let upcall_ptr = upcall as unsafe extern "C" fn(u32, u32, u32, Register);
        with_kernel_data(|kernel_data| {
            let kernel_data = kernel_data.unwrap();

            // Verify the upcall was not queued.
            assert!(kernel_data.upcall_queue.is_empty());

            // Register a non-null upcall.
            kernel_data.drivers.get_mut(&1).unwrap().upcalls.insert(
                2,
                Upcall {
                    fn_pointer: Some(upcall_ptr),
                    data: 1111usize.into(),
                },
            );
        });
        // Call schedule again. This should schedule the upcall.
        assert_eq!(mock_driver.share_ref.schedule_upcall(2, (3, 4, 5)), Ok(()));
        with_kernel_data(|kernel_data| {
            let kernel_data = kernel_data.unwrap();

            // Verify the upcall was queued.
            assert_eq!(kernel_data.upcall_queue.len(), 1);
            let upcall_queue_entry = kernel_data.upcall_queue.front().expect("Upcall not queued");
            assert_eq!(upcall_queue_entry.args, (3, 4, 5));
            assert_eq!(
                upcall_queue_entry.id,
                UpcallId {
                    driver_num: 1,
                    subscribe_num: 2
                }
            );
            assert!(upcall_queue_entry.upcall.fn_pointer == Some(upcall_ptr));
            let data: usize = upcall_queue_entry.upcall.data.into();
            assert_eq!(data, 1111);

            // Register a non-null upcall.
            kernel_data.drivers.get_mut(&1).unwrap().upcalls.insert(
                2,
                Upcall {
                    fn_pointer: Some(upcall_ptr),
                    data: 2222u32.into(),
                },
            );
        });
        // Call schedule again. This should schedule another upcall, after the
        // first.
        assert_eq!(
            mock_driver.share_ref.schedule_upcall(2, (30, 40, 50)),
            Ok(())
        );
        with_kernel_data(|kernel_data| {
            let kernel_data = kernel_data.unwrap();

            // Very the upcall was queued.
            assert_eq!(kernel_data.upcall_queue.len(), 2);
            let front_queue_entry = kernel_data.upcall_queue.front().expect("Upcall not queued");
            assert_eq!(front_queue_entry.args, (3, 4, 5));
            assert_eq!(
                front_queue_entry.id,
                UpcallId {
                    driver_num: 1,
                    subscribe_num: 2
                }
            );
            assert!(front_queue_entry.upcall.fn_pointer == Some(upcall_ptr));
            let front_data: usize = front_queue_entry.upcall.data.into();
            assert_eq!(front_data, 1111);
            let back_queue_entry = kernel_data.upcall_queue.back().expect("Upcall not queued");
            assert_eq!(back_queue_entry.args, (30, 40, 50));
            assert_eq!(
                back_queue_entry.id,
                UpcallId {
                    driver_num: 1,
                    subscribe_num: 2
                }
            );
            assert!(back_queue_entry.upcall.fn_pointer == Some(upcall_ptr));
            let back_data: usize = back_queue_entry.upcall.data.into();
            assert_eq!(back_data, 2222);
        });
    }
}
