/// Implements `share::List` and `share::SplittableHandle` on tuples of various
/// sizes.
use crate::share::{Handle, List, SplittableHandle};

// Implement List and SplittableHandle on empty tuples, because tuple_impl only
// works for tuples of size 1 or greater. Empty lists may be useful in generic
// and/or macro contexts.
impl List for () {}

impl<'handle> SplittableHandle<'handle> for () {
    type SplitHandles = ();

    fn split(_handle: Handle<'handle, ()>) {}
}

// Provides `share::List` and `share::SplittableHandle` impls for tuples of a
// specific size. tuple_impls! must be provided with a list of names, and will
// generate impls for tuples of the given length.
macro_rules! tuple_impls {
    ($($name:ident),*) => {
        impl<$($name: List),*> List for ($($name),*,) {}

        impl<'handle, $($name: List + 'handle),*> SplittableHandle<'handle> for ($($name),*,) {
            type SplitHandles = ($(Handle<'handle, $name>),*,);

            fn split(handle: Handle<'handle, Self>) -> Self::SplitHandles {
                // Safety: handle guarantees that an instance of Self exists and
                // will be cleaned up before it becomes invalid. Self is a
                // tuple, and the types we are changing handle into are elements
                // of that tuple, so when the tuple is cleaned up they will be
                // cleaned up as well.
                ($(unsafe { handle.change_type::<$name>() }),*,)
            }
        }
    }
}

// Recursively calls tuple_impls for all tuples of a given length or shorter
// (except the empty tuple, which tuple_impls doesn't support).
macro_rules! impl_recursive {
    // Base case: if provided no names, do nothing.
    () => {};

    // Recursive case. Calls tuple_impls, then call ourselves with one less
    // name.
    ($head_name:ident$(, $($tail_names:ident),*)?) => {
        tuple_impls!($head_name$(, $($tail_names),*)?);
        impl_recursive!($($($tail_names),*)?);
    };
}

// Because List depends on Default, which is only implemented for tuples of <=
// 12 elements, we can only implement List for tuples of up to length 12.
impl_recursive!(A, B, C, D, E, F, G, H, I, J, K, L);
