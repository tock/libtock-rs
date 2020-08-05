/// An individual value that has been shared with the kernel using the `allow`
/// system call.
// Allowed's implementation does not directly use the 'b lifetime. Platform uses
// 'b to prevent the Allowed from accessing the buffer after the buffer becomes
// invalid.
// Allowed requires T to be Copy due to concerns about the semantics of
// non-copyable types in shared memory as well as concerns about unexpected
// behavior with Drop types. See the following PR discussion for more
// information: https://github.com/tock/libtock-rs/pull/222
pub struct Allowed<'b, T: Copy + 'b> {
    // Safety properties:
    //   1. `buffer` remains valid and usable for the lifetime of this Allowed
    //      instance.
    //   2. Read and write accesses of `buffer`'s pointee must be performed as a
    //      volatile operation, as the kernel may mutate buffer's pointee at any
    //      time.
    //   3. The value `buffer` points to may have an arbitrary bit pattern in
    //      it, so reading from `buffer` is only safe if the type contained
    //      within is AllowReadable.
    buffer: core::ptr::NonNull<T>,

    // We need to use the 'b lifetime in Allowed to prevent an "unused lifetime"
    // compiler error. We use it here. Note that the phantom data must be an
    // &mut rather than a shared reference in order to make Allowed invariant
    // with respect to T. Invariance is required because Allowed allows us to
    // mutate the value in buffer, and invariance is the required property to do
    // so without allowing callers to produce dangling references. This is
    // documented at https://doc.rust-lang.org/nomicon/subtyping.html.
    _phantom: core::marker::PhantomData<&'b mut T>,
}

// Allowed's API is based on that of core::cell::Cell, but removes some methods
// that are not safe for use with shared memory.
//
// Internally, Allowed performs accesses to the shared memory using volatile
// reads and writes through raw pointers. We avoid constructing references to
// shared memory because that leads to undefined behavior (there is some
// background on this in the following discussion:
// https://github.com/rust-lang/unsafe-code-guidelines/issues/33). Tock runs on
// single-threaded platforms, some of which lack atomic instructions, so we only
// need to be able to deconflict races between the kernel (which will never
// interrupt an instruction's execution) and this process. Therefore volatile
// accesses are sufficient to deconflict races.
impl<'b, T: Copy + 'b> Allowed<'b, T> {
    // Allowed can only be constructed by the Platform. It is constructed after
    // the `allow` system call, and as such must accept a raw pointer rather
    // than a reference. The caller must make sure the following are true:
    // 1. buffer points to a valid instance of type T
    // 2. There are no references to buffer's pointee
    // 3. buffer remains usable until the Allowed's lifetime has ended.
    #[allow(dead_code)] // TODO: Remove when Platform is implemented
    pub(crate) unsafe fn new(buffer: core::ptr::NonNull<T>) -> Allowed<'b, T> {
        Allowed {
            buffer,
            _phantom: core::marker::PhantomData,
        }
    }

    // Sets the value in the buffer.
    pub fn set(&self, value: T) {
        unsafe {
            core::ptr::write_volatile(self.buffer.as_ptr(), value);
        }
    }
}

impl<'b, T: crate::AllowReadable + Copy + 'b> Allowed<'b, T> {
    pub fn replace(&self, value: T) -> T {
        let current = unsafe { core::ptr::read_volatile(self.buffer.as_ptr()) };
        unsafe {
            core::ptr::write_volatile(self.buffer.as_ptr(), value);
        }
        current
    }

    pub fn get(&self) -> T {
        unsafe { core::ptr::read_volatile(self.buffer.as_ptr()) }
    }
}

impl<'b, T: crate::AllowReadable + Copy + Default + 'b> Allowed<'b, T> {
    pub fn take(&self) -> T {
        self.replace(T::default())
    }
}
