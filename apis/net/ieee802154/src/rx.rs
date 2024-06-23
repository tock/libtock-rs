use core::marker::PhantomData;

use super::*;

/// Maximum length of a MAC frame.
const MAX_MTU: usize = 127;

#[derive(Debug)]
#[repr(C)]
pub struct Frame {
    pub header_len: u8,
    pub payload_len: u8,
    pub mic_len: u8,
    pub body: [u8; MAX_MTU],
}

const EMPTY_FRAME: Frame = Frame {
    header_len: 0,
    payload_len: 0,
    mic_len: 0,
    body: [0; MAX_MTU],
};

/// The ring buffer that is shared with kernel using allow-rw syscall, with kernel acting
/// as a producer of frames and we acting a consumer.

/// The `N` parameter specifies the capacity of the buffer in number of frames.
/// Unfortunately, due to a design flaw of the ring buffer, it can never be fully utilised,
/// as it's impossible to distinguish an empty buffer from a full one. The kernel code
/// actually uses up to `N - 1` slots, and then starts overwriting old frames with
/// new ones. Remember to specify `N` as `F + 1`, where `F` is the maximum expected number
/// of frames received in short succession.
///
/// Given the non-deterministic nature of upcalls, the userprocess must carefully
/// handle receiving upcalls. There exists a risk of dropping 15.4 packets while
/// reading from the ring buffer (as the ring buffer is unallowed while reading).
/// This can be handled by utilizing two ring buffers and alternating which
/// belongs to the kernel and which is being read from. This is done by [RxBufferAlternatingOperator].
/// Alternatively, the user can also utilize a single ring buffer if dropped frames may be permissible.
/// This is done by [RxSingleBufferOperator].
#[derive(Debug)]
#[repr(C)]
pub struct RxRingBuffer<const N: usize> {
    /// From where the next frame will be read by process.
    /// Updated by process only.
    read_index: u8,
    /// Where the next frame will be written by kernel.
    /// Updated by kernel only.
    write_index: u8,
    /// Slots for received frames.
    frames: [Frame; N],
}

impl<const N: usize> RxRingBuffer<N> {
    /// Creates a new [RxRingBuffer] that can be used to receive frames into.
    pub const fn new() -> Self {
        Self {
            read_index: 0,
            write_index: 0,
            frames: [EMPTY_FRAME; N],
        }
    }

    fn as_mut_byte_slice(&mut self) -> &mut [u8] {
        // SAFETY: any byte value is valid for any byte of Self,
        // as well as for any byte of [u8], so casts back and forth
        // cannot break the type system.
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut Self as *mut u8,
                core::mem::size_of::<Self>(),
            )
        }
    }

    fn has_frame(&self) -> bool {
        self.read_index != self.write_index
    }

    fn next_frame(&mut self) -> &mut Frame {
        let frame = self.frames.get_mut(self.read_index as usize).unwrap();
        self.read_index = (self.read_index + 1) % N as u8;
        frame
    }
}

pub trait RxOperator {
    /// Receive one new frame.
    ///
    /// Logically pop one frame out of the ring buffer and provide mutable access to it.
    /// If no frame is ready for reception, yield_wait to kernel until one is available.
    fn receive_frame(&mut self) -> Result<&mut Frame, ErrorCode>;
}

/// Safe encapsulation that can receive frames from the kernel using a single ring buffer.
/// See [RxRingBuffer] for more information.
///
/// This operator can lose some frames: if a frame is received in the kernel when
/// the app is examining its received frames (and hence has its buffer unallowed),
/// then the frame can be lost. See [RxBufferAlternatingOperator] for an alternative.
pub struct RxSingleBufferOperator<'buf, const N: usize, S: Syscalls, C: Config = DefaultConfig> {
    buf: &'buf mut RxRingBuffer<N>,
    s: PhantomData<S>,
    c: PhantomData<C>,
}

impl<'buf, const N: usize, S: Syscalls, C: Config> RxSingleBufferOperator<'buf, N, S, C> {
    /// Creates a new [RxSingleBufferOperator] that can be used to receive frames.
    pub fn new(buf: &'buf mut RxRingBuffer<N>) -> Self {
        Self {
            buf,
            s: PhantomData,
            c: PhantomData,
        }
    }
}
impl<'buf, const N: usize, S: Syscalls, C: Config> RxOperator
    for RxSingleBufferOperator<'buf, N, S, C>
{
    fn receive_frame(&mut self) -> Result<&mut Frame, ErrorCode> {
        if self.buf.has_frame() {
            Ok(self.buf.next_frame())
        } else {
            // If no frame is there, wait until one comes, then return it.

            Ieee802154::<S, C>::receive_frame_single_buf(self.buf)?;

            // Safety: kernel schedules an upcall iff a new frame becomes available,
            // i.e. when it increments `read_index`.
            Ok(self.buf.next_frame())
        }
    }
}

// Reception
impl<S: Syscalls, C: Config> Ieee802154<S, C> {
    fn receive_frame_single_buf<const N: usize>(
        buf: &mut RxRingBuffer<N>,
    ) -> Result<(), ErrorCode> {
        let called: Cell<Option<(u32,)>> = Cell::new(None);
        share::scope::<
            (
                AllowRw<_, DRIVER_NUM, { allow_rw::READ }>,
                Subscribe<_, DRIVER_NUM, { subscribe::FRAME_RECEIVED }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_rw, subscribe) = handle.split();
            S::allow_rw::<C, DRIVER_NUM, { allow_rw::READ }>(allow_rw, buf.as_mut_byte_slice())?;
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::FRAME_RECEIVED }>(subscribe, &called)?;

            loop {
                S::yield_wait();
                if let Some((_lqi,)) = called.get() {
                    // At least one frame was received.
                    return Ok(());
                }
            }
        })
    }
}

// This module is to protect RxRingBufferInKernel private API.
mod alternate_receive_buffers {
    use super::*;

    pub(super) struct RxRingBufferInKernel<'buf, const N: usize, S: Syscalls, C: Config> {
        lent_buf: *mut RxRingBuffer<N>,
        covariance_phantom: PhantomData<&'buf mut RxRingBuffer<N>>,
        sc: PhantomData<(S, C)>,
    }

    impl<'buf, const N: usize, S: Syscalls, C: Config> Drop for RxRingBufferInKernel<'buf, N, S, C> {
        // Unshares the buffer to prevent kernel accessing no longer valid memory.
        fn drop(&mut self) {
            share::scope::<AllowRw<_, DRIVER_NUM, { allow_rw::READ }>, _, _>(|handle| {
                let _ = S::allow_rw::<C, DRIVER_NUM, { allow_rw::READ }>(handle, &mut []);
            });
        }
    }

    impl<'buf, const N: usize, S: Syscalls, C: Config> RxRingBufferInKernel<'buf, N, S, C> {
        fn new(buf: &mut RxRingBuffer<N>) -> Self {
            Self {
                lent_buf: buf as *mut RxRingBuffer<N>,
                covariance_phantom: PhantomData,
                sc: PhantomData,
            }
        }

        pub(super) fn share_initial(buf: &mut RxRingBuffer<N>) -> Result<Self, ErrorCode> {
            Self::share_unscoped(buf.as_mut_byte_slice())?;
            Ok(Self::new(buf))
        }

        fn share_unscoped(buf: &mut [u8]) -> Result<(), ErrorCode> {
            let allow_rw = platform::AllowRw::<S, DRIVER_NUM, { allow_rw::READ }>::default();

            // Safety: The buffer being allowed here is going to be enclosed in an opaque type
            // until it's unallowed again. This prevents concurrent access to the buffer by process and kernel.
            let allow_rw_handle = unsafe { share::Handle::new(&allow_rw) };

            // This does not really share the buffer with kernel, but for
            // easier invariant later let's assume it's shared since now.
            S::allow_rw::<C, DRIVER_NUM, { allow_rw::READ }>(allow_rw_handle, buf)?;

            // This is crucial. This prevents unallowing the buffer at the end of scope.
            // Thanks to that, some buffer is constantly allowed for kernel to write there,
            // preventing frame loss at any point.
            core::mem::forget(allow_rw);

            Ok(())
        }

        /// Swaps the two buffers.
        /// Shares with the kernel the alternate buffer, at the same time unsharing this buffer.
        pub(super) fn swap(&mut self, buf: &mut &mut RxRingBuffer<N>) -> Result<(), ErrorCode> {
            Self::share_unscoped(buf.as_mut_byte_slice())?;

            let allowed_buf = Self::new(buf);

            // SAFETY: `lent_buf` was created from a mutable reference, so recreation of that mutable
            // reference is sound. Lifetimes and aliasing rules were enforced all the time by
            // `covariance_phantom`, which by covariance with the original mutable reference
            // kept it valid.
            let returned_buf = unsafe { &mut *self.lent_buf };

            let no_longer_shared_kernel_buf = core::mem::replace(self, allowed_buf);

            // This is crucial. This disarms the drop mechanism that would unshare the newly shared alternate buffer.
            core::mem::forget(no_longer_shared_kernel_buf);

            *buf = returned_buf;

            Ok(())
        }
    }
}
use alternate_receive_buffers::RxRingBufferInKernel;

/// Safe encapsulation that can receive frames from the kernel using a pair of ring buffers.
/// See [RxRingBuffer] for more information.
///
/// This operator won't lose frames: if a frame is received in the kernel when
/// the app is examining its received frames is one of the buffers, then the other buffer
/// is allowed, so the frame won't be lost. This comes with cost: one has to pay twice
/// as much memory to keep a pair of buffers.
pub struct RxBufferAlternatingOperator<'buf, const N: usize, S: Syscalls, C: Config = DefaultConfig>
{
    buf_ours: &'buf mut RxRingBuffer<N>,
    buf_kernels: RxRingBufferInKernel<'buf, N, S, C>,
    s: PhantomData<S>,
    c: PhantomData<C>,
}

impl<'buf, const N: usize, S: Syscalls, C: Config> RxBufferAlternatingOperator<'buf, N, S, C> {
    /// Creates a new [RxSingleBufferOperator] that can be used to receive frames.
    pub fn new(
        buf1: &'buf mut RxRingBuffer<N>,
        buf2: &'buf mut RxRingBuffer<N>,
    ) -> Result<Self, ErrorCode> {
        Ok(Self {
            buf_ours: buf1,
            buf_kernels: RxRingBufferInKernel::share_initial(buf2)?,
            s: PhantomData,
            c: PhantomData,
        })
    }
}
impl<'buf, const N: usize, S: Syscalls, C: Config> RxOperator
    for RxBufferAlternatingOperator<'buf, N, S, C>
{
    /// Receive a new frame.
    fn receive_frame(&mut self) -> Result<&mut Frame, ErrorCode> {
        // First check if there are frames in buf_ours. If so, return first of them.
        if self.buf_ours.has_frame() {
            return Ok(self.buf_ours.next_frame());
        }

        let called: Cell<Option<(u32,)>> = Cell::new(None);
        // If buf_ours is empty, subscribe to upcalls at this point not to lose any upcall.
        share::scope::<(Subscribe<_, DRIVER_NUM, { subscribe::FRAME_RECEIVED }>,), _, _>(|handle| {
            let (subscribe,) = handle.split();
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::FRAME_RECEIVED }>(subscribe, &called)?;

            // Swap the buffers and look again in buf_ours.
            self.buf_kernels.swap(&mut self.buf_ours)?;

            // If there's a frame, return it.
            if self.buf_ours.has_frame() {
                return Ok(self.buf_ours.next_frame());
            }

            // If nothing's there, yield wait.
            loop {
                S::yield_wait();
                if let Some((_lqi,)) = called.get() {
                    // At least one frame was received.
                    break;
                }
            }

            // Swap buffers again. Now there must be a frame in buf_ours.
            self.buf_kernels.swap(&mut self.buf_ours)?;

            Ok(self.buf_ours.next_frame())
        })
    }
}
