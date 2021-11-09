/// `syscall_scope` creates a scope in which code can share objects with the
/// kernel (through the Allow and Subscribe system calls). At the end of the
/// scope, the shared objects are automatically cleaned up (buffers shared using
/// Allow are retrieved, and upcalls shared using Subscribe are removed).
pub fn syscall_scope<'scope, SL: ShareList<'scope>, Output, F: FnOnce(&SL) -> Output>(
    fcn: F,
) -> Output {
    // Safety: We do not move syscall_list, so it is always destructed at the
    // end of the current scope (including in the event of an unwinding panic).
    let syscall_list = unsafe { SL::new() };
    fcn(&syscall_list)
}

// A list of zero or more objects that may be shared with the kernel. Code that
// creates ShareList instances must promise to run the ShareList's destructor
// before its scope ends -- this allows the ShareList to revoke the kernel's
// access to the objects before they are deallocated.
pub trait ShareList<'scope> {
    // Safety: The caller must guarantee that Drop::drop is called on this
    // ShareList before the 'scope lifetime ends.
    //
    // Note that it is okay to not call Drop::drop if the 'scope lifetime will
    // not end (e.g. 'scope is 'static or the code is guaranteed to loop
    // forever).
    unsafe fn new() -> Self;
}

// Implement ShareList for the empty tuple (the impl_share_list macro doesn't
// work with 0 type arguments). This may be useful in generic and/or macro
// contexts.
impl<'scope> ShareList<'scope> for () {
    unsafe fn new() {}
}

// Implements ShareList on a tuple of ShareLists. Takes a list of names as its
// arguments; the names are used as type parameters in the generic
// implementation. The number of names provided controls the size of the tuples
// that ShareList is implemented on.
macro_rules! impl_share_list {
    ($($name:ident),* $(,)?) => {
        impl<'scope, $($name: ShareList<'scope>),*> ShareList<'scope> for ($($name),*,) {
            unsafe fn new() -> Self {
                // Safety: This caller of this function guarantees the returned
                // Self will be dropped before 'scope ends. Because Self is a
                // tuple, when it is dropped it will drop all its elements as
                // well.
                ($(unsafe { $name::new() }),*,)
            }
        }
    }
}

// Recursively implements ShareList on all tuples of less than or equal to a
// certain length (except the 0-length tuple (), which impl_share_list doesn't
// support).
macro_rules! impl_recursive {
    // Base case for the recursion: if no name is passed, do nothing.
    () => {};
    ($head_name:ident$(, $($tail_names:ident),*)?) => {
        impl_share_list!($head_name, $($($tail_names),*)?);
        impl_recursive!($($($tail_names),*)?);
    };
}

impl_recursive!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

#[cfg(test)]
mod tests {
    use super::*;

    // InstanceCounter implements ShareList, and counts the number of instances
    // that exist in the current thread.
    struct InstanceCounter;

    std::thread_local! {static INSTANCE_COUNT: core::cell::Cell<u64> = core::cell::Cell::new(0)}

    impl Drop for InstanceCounter {
        fn drop(&mut self) {
            INSTANCE_COUNT.with(|cell| cell.set(cell.get() - 1));
        }
    }

    impl<'scope> ShareList<'scope> for InstanceCounter {
        unsafe fn new() -> InstanceCounter {
            INSTANCE_COUNT.with(|cell| cell.set(cell.get() + 1));
            InstanceCounter
        }
    }

    // This test case will not compile unless ShareList is implemented for
    // certain tuple sizes.
    #[test]
    fn tuple_impls() {
        fn require_share_list<'scope, SL: ShareList<'scope>>() {}

        require_share_list::<()>();
        require_share_list::<(InstanceCounter,)>();
        require_share_list::<(InstanceCounter, ())>();
        #[rustfmt::skip]
        require_share_list::<((), (), (), (), (),
                              (), (), (), (), (),
                              (), (), (), (), (),
                              (), (), (), (), (),
                              (), (), (), (), (), InstanceCounter)>();
    }

    // Verifies that syscall_scope correctly constructs and cleans up the
    // ShareList.
    #[test]
    fn syscall_scope_cleanup() {
        // INSTANCE_COUNT *should* be 0 here, but make sure it's zero in case
        // another test case leaked an instance.
        INSTANCE_COUNT.with(|cell| cell.set(0));
        let out = syscall_scope(|_scope: &(InstanceCounter, (), InstanceCounter)| {
            assert_eq!(INSTANCE_COUNT.with(|cell| cell.get()), 2);
            42
        });
        assert_eq!(INSTANCE_COUNT.with(|cell| cell.get()), 0);
        assert_eq!(out, 42);
    }
}
