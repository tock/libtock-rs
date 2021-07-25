/// Adds an upcall to the upcall queue, to be invoked during a future Yield
/// call. Operate's on this thread's `fake::Kernel`.
///
/// Like the real kernel, this does nothing and returns success if there is no
/// upcall or the upcall is a null upcall.
pub fn schedule(
    driver_number: u32,
    subscribe_number: u32,
    args: (u32, u32, u32),
) -> Result<(), ScheduleError> {
    crate::kernel_data::with_kernel_data(|kernel_data| {
        let kernel_data = kernel_data.ok_or(ScheduleError::NoKernel)?;
        let driver_data = kernel_data
            .drivers
            .get(&driver_number)
            .ok_or(ScheduleError::NoDriver(driver_number))?;
        if subscribe_number >= driver_data.num_upcalls {
            return Err(ScheduleError::TooLargeSubscribeNumber {
                num_upcalls: driver_data.num_upcalls,
                requested: subscribe_number,
            });
        }
        let upcall = match driver_data.upcalls.get(&subscribe_number) {
            Some(&upcall) => upcall,
            None => return Ok(()),
        };
        // Don't bother queueing a null upcall, as they don't do anything when
        // invoked anyway, and the core kernel does not queue them either.
        if upcall.is_null() {
            return Ok(());
        }
        kernel_data.upcall_queue.push_back(UpcallQueueEntry {
            args,
            id: UpcallId {
                driver_number,
                subscribe_number,
            },
            upcall,
        });
        Ok(())
    })
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ScheduleError {
    #[error("Driver number {0} does not exist.")]
    NoDriver(u32),

    #[error("No fake::Kernel exists in this thread")]
    NoKernel,

    #[error("Upcall number {requested} too large, expected < {num_upcalls}.")]
    TooLargeSubscribeNumber { num_upcalls: u32, requested: u32 },
}

/// Raw upcall data, as it was passed to Subscribe. This upcall is not
/// guaranteed to still be valid.
#[derive(Clone, Copy)]
pub struct Upcall {
    pub fn_pointer: Option<unsafe extern "C" fn(u32, u32, u32, libtock_platform::Register)>,
    pub data: libtock_platform::Register,
}

impl Upcall {
    /// Returns true if this is a null callback, false otherwise.
    pub fn is_null(&self) -> bool {
        self.fn_pointer.is_none()
    }
}

// The type of the upcall queue in KernelData. Contains queued upcalls, which
// are waiting to be invoked during a Yield call.
//
// Based on this discussion:
// https://mailman.stanford.edu/pipermail/tock-version2/2020-November/000025.html
// this queue is a FIFO queue. New entries should be pushed to the back of the
// queue, and Yield should invoke upcalls starting with the front of the queue.
//
// A note on performance: When an upcall is replaced via Subscribe and becomes
// invalid, Subscribe has to iterate through this queue and remove all instances
// of that upcall. That takes linear time in the length of the queue, so it can
// be slow if the queue is long. A long queue is unrealistic, so we shouldn't
// need a long queue in test cases, so that is acceptable. There are alternative
// data structures that avoid that slowdown, but they are more complex and
// likely slower in the common case.
pub(crate) type UpcallQueue = std::collections::VecDeque<crate::upcall::UpcallQueueEntry>;

// An entry in the fake kernel's upcall queue.
pub(crate) struct UpcallQueueEntry {
    pub args: (u32, u32, u32),
    pub id: UpcallId,
    pub upcall: Upcall,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) struct UpcallId {
    driver_number: u32,
    subscribe_number: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDriver;
    impl crate::fake::Driver for MockDriver {
        fn id(&self) -> u32 {
            1
        }
        fn num_upcalls(&self) -> u32 {
            10
        }
        fn command(&self, _: u32, _: u32, _: u32) -> libtock_platform::CommandReturn {
            crate::command_return::failure(libtock_platform::ErrorCode::NoSupport)
        }
    }

    #[test]
    fn schedule_errors() {
        use ScheduleError::{NoDriver, NoKernel, TooLargeSubscribeNumber};

        assert_eq!(schedule(1, 2, (3, 4, 5)), Err(NoKernel));

        let kernel = crate::fake::Kernel::new();
        assert_eq!(schedule(1, 2, (3, 4, 5)), Err(NoDriver(1)));

        kernel.add_driver(&std::rc::Rc::new(MockDriver));
        assert_eq!(
            schedule(1, 10, (3, 4, 5)),
            Err(TooLargeSubscribeNumber {
                num_upcalls: 10,
                requested: 10
            })
        );
    }

    #[test]
    fn schedule_success() {
        let kernel = crate::fake::Kernel::new();
        kernel.add_driver(&std::rc::Rc::new(MockDriver));

        // Call schedule with no registered upcall.
        assert_eq!(schedule(1, 2, (3, 4, 5)), Ok(()));
        crate::kernel_data::with_kernel_data(|kernel_data| {
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
        assert_eq!(schedule(1, 2, (3, 4, 5)), Ok(()));
        unsafe extern "C" fn upcall(_: u32, _: u32, _: u32, _: libtock_platform::Register) {}
        crate::kernel_data::with_kernel_data(|kernel_data| {
            let kernel_data = kernel_data.unwrap();

            // Very the upcall was not queued.
            assert!(kernel_data.upcall_queue.is_empty());

            // Register a non-null upcall.
            kernel_data.drivers.get_mut(&1).unwrap().upcalls.insert(
                2,
                Upcall {
                    fn_pointer: Some(upcall),
                    data: 1111usize.into(),
                },
            );
        });
        // Call schedule again. This should schedule the upcall.
        assert_eq!(schedule(1, 2, (3, 4, 5)), Ok(()));
        crate::kernel_data::with_kernel_data(|kernel_data| {
            let kernel_data = kernel_data.unwrap();

            // Verify the upcall was queued.
            assert_eq!(kernel_data.upcall_queue.len(), 1);
            let upcall_queue_entry = kernel_data.upcall_queue.front().expect("Upcall not queued");
            assert_eq!(upcall_queue_entry.args, (3, 4, 5));
            assert_eq!(
                upcall_queue_entry.id,
                UpcallId {
                    driver_number: 1,
                    subscribe_number: 2
                }
            );
            assert!(upcall_queue_entry.upcall.fn_pointer == Some(upcall));
            let data: usize = upcall_queue_entry.upcall.data.into();
            assert_eq!(data, 1111);

            // Register a non-null upcall.
            kernel_data.drivers.get_mut(&1).unwrap().upcalls.insert(
                2,
                Upcall {
                    fn_pointer: Some(upcall),
                    data: 2222u32.into(),
                },
            );
        });
        // Call schedule again. This should schedule another upcall, after the
        // first.
        assert_eq!(schedule(1, 2, (30, 40, 50)), Ok(()));
        crate::kernel_data::with_kernel_data(|kernel_data| {
            let kernel_data = kernel_data.unwrap();

            // Very the upcall was queued.
            assert_eq!(kernel_data.upcall_queue.len(), 2);
            let front_queue_entry = kernel_data.upcall_queue.front().expect("Upcall not queued");
            assert_eq!(front_queue_entry.args, (3, 4, 5));
            assert_eq!(
                front_queue_entry.id,
                UpcallId {
                    driver_number: 1,
                    subscribe_number: 2
                }
            );
            assert!(front_queue_entry.upcall.fn_pointer == Some(upcall));
            let front_data: usize = front_queue_entry.upcall.data.into();
            assert_eq!(front_data, 1111);
            let back_queue_entry = kernel_data.upcall_queue.back().expect("Upcall not queued");
            assert_eq!(back_queue_entry.args, (30, 40, 50));
            assert_eq!(
                back_queue_entry.id,
                UpcallId {
                    driver_number: 1,
                    subscribe_number: 2
                }
            );
            assert!(back_queue_entry.upcall.fn_pointer == Some(upcall));
            let back_data: usize = back_queue_entry.upcall.data.into();
            assert_eq!(back_data, 2222);
        });
    }
}
