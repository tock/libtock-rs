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

    /// # Safety
    /// An upcall may only be invoked if it is still active. As described in TRD
    /// 104, an upcall is still active if it has not been replaced by another
    /// Subscribe call with the same upcall ID. All upcalls in the upcall queue
    /// in KernelData are active.
    pub unsafe fn invoke(&self, args: (u32, u32, u32)) {
        if let Some(fn_pointer) = self.fn_pointer {
            unsafe {
                fn_pointer(args.0, args.1, args.2, self.data);
            }
        }
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
pub(crate) type UpcallQueue = std::collections::VecDeque<UpcallQueueEntry>;

// An entry in the fake kernel's upcall queue.
pub(crate) struct UpcallQueueEntry {
    pub args: (u32, u32, u32),
    pub id: UpcallId,
    pub upcall: Upcall,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct UpcallId {
    pub driver_num: u32,
    pub subscribe_num: u32,
}
