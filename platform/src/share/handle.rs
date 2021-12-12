use crate::share::List;
use core::marker::PhantomData;

/// A `Handle`'s existence indicates two things:
///
/// 1. The specified List exists.
/// 2. The specified List will be cleaned up (via `Drop::drop`) rather than
///    leaked or forgotten.
///
/// `Handle`s are used to call system calls, and should generally be created by
/// using `share::scope`.
pub struct Handle<'handle, L: List> {
    // Handle acts like a &'handle L, so set its variance accordingly. Note that
    // the most important lifetime -- the lifetime the share can live for -- is
    // a parameter of L, not Handle.
    //
    // Additionally, _list is a private member, which prevents code outside this
    // module from constructing a Handle without calling Handle's constructors.
    _list: PhantomData<&'handle L>,
}

// We can't #[derive(Clone, Copy)] because derive's implementations of Clone and
// Copy have `L: Clone` and `L: Copy` constraints, respectively. We don't want
// those constraints, so we manually implement Clone and Copy.
impl<'handle, L: List> Clone for Handle<'handle, L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'handle, L: List> Copy for Handle<'handle, L> {}

impl<'handle, L: List> Handle<'handle, L> {
    /// Constructs a new `Handle`, typing its lifetime to the provided list.
    ///
    /// # Safety
    /// The calling code must guarantee that `Drop::drop` is called on `_list`'s
    /// pointee before `_list`'s pointee becomes invalid. In other words,
    /// `_list`'s pointee may not be forgotten or leaked.
    pub unsafe fn new(_list: &'handle L) -> Self {
        Handle { _list: PhantomData }
    }

    /// Converts this `Handle` to a `Handle` of a different type. Used to create
    /// a handle to a sub-object of this handle's `L`.
    ///
    /// # Safety
    /// The calling code must guarantee that an `Other` exists, and that
    /// `Drop::drop` will be called on the `Other` before the `Other` becomes
    /// invalid. In general, the `Other` should be contained inside the `L`, so
    /// `Drop::drop` is executed on the `Other` when the `L` is dropped.
    pub unsafe fn change_type<Other: List>(self) -> Handle<'handle, Other> {
        Handle { _list: PhantomData }
    }

    /// Splits this `Handle` into a list of handles to sub-lists of `L`. Used
    /// when `L` is a tuple of shares, to obtain handles to the individual
    /// shares.
    pub fn split(self) -> L::SplitHandles
    where
        L: SplittableHandle<'handle>,
    {
        L::split(self)
    }
}

/// A trait implemented by `share::List`s that can be divided into sub-lists.
/// `SplittableHandle` should not be used directly; instead, callers should use
/// `Handle::split`.
pub trait SplittableHandle<'handle>: List {
    /// SplitHandles should be a tuple of Handle types, i.e. (Handle<>,
    /// Handle<>, ...).
    type SplitHandles;

    /// Split the specified handle into sub-handles. Implementations of `split`
    /// should use `Handle::change_type` to create the sub-handles.
    fn split(handle: Handle<'handle, Self>) -> Self::SplitHandles;
}
