//! `async_traits` provides the `AsyncCall`, `Callback`, and `StaticCallback`
//! traits. These traits form the basis for `libtock_core`'s asynchronous API
//! structure.
//!
//! In general, an `AsyncCall` is used to initiate an asynchronous operation.
//! When the operation is complete, a `Callback` or `StaticCall` is used to
//! inform the client of the operation's completion. Clients may then issue
//! additional `AsyncCall` invocations from within the response callback.
//!
//! Note that callbacks should not be issued from within the `AsyncCall` call;
//! doing so causes reentrancy and stack utilization issues. This is enforced
//! through use of the CallbackContext type parameter.
//!
//! These traits are designed to be implemented on zero sized types (ZSTs) that
//! refer indirectly to the objects that implement the corresponding
//! functionality. As a result, they require `Copy` and take `self` by value. In
//! many cases, these traits will be implemented on references to objects rather
//! than on the objects themselves. For example, a `Console` driver would
//! implement `AsyncCall` on `&Console`, not on `Console` itself, as `Console`
//! would not be a `Copy` type.

/// Represents a call that starts an asynchronous operation. The `start` request
/// can return a payload synchronously; this may be used to return a "failed to
/// start"-style error message.
pub trait AsyncCall<Request, SyncResponse>: Copy {
    fn start(self, request: Request) -> SyncResponse;
}

/// A callback reporting the results of an asynchronous operation. Like
/// AsyncCall, this callback may carry data, and as such `Callback` may be
/// implemented on reference types.
pub trait Callback<AsyncResponse>: Copy {
    fn callback(self, context: CallbackContext, response: AsyncResponse);
}

/// A marker type indicating this method executes as a callback. In this case,
/// callbacks refer either to kernel callbacks (those resulting from a
/// `subscribe` sysem call) or to deferred callbacks. In both cases, the
/// `Platform` implementation provides the `CallbackContext`.
///
/// `CallbackContext` is `Copy` so it is easy to pass into function invocations.
// The lifetime parameter 'c exists to prevent a CallbackContext from being
// moved into a location that outlives the callback's execution.
#[derive(Clone, Copy)]
pub struct CallbackContext<'c> {
    // This serves two purposes. First, it is `pub(crate)`, so user code cannot
    // construct the CallbackContext. Second, it uses the lifetime parameter to
    // avoid an "unused lifetime parameter" error.
    pub(crate) _private: core::marker::PhantomData<&'c ()>,
}

/// A variation of callback that is not allowed to carry data. This conveys the
/// callback entirely via the type system. This is used in places that cannot
/// carry runtime data, such as the system call API, but generally requires the
/// client to store the data it needs in a `static` location. As such, APIs
/// should prefer to support `Callback` wherever possible.
pub trait StaticCallback<AsyncResponse> {
    fn callback(context: CallbackContext, response: AsyncResponse);
}
