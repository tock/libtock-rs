//! `share` contains tools for safely sharing objects (such as buffers and
//! upcalls) with the Tock kernel.

mod handle;
mod tuple_impls;

pub use handle::{Handle, SplittableHandle};

/// Creates a scope in which objects may safely be shared with the kernel.
pub fn scope<L: List, Output, F: FnOnce(Handle<L>) -> Output>(fcn: F) -> Output {
    let list = Default::default();
    // Safety: We do not move the L out of the `list` variable. The `list`
    // variable will be dropped at the end of the scope, immediately before the
    // L becomes invalid.
    fcn(unsafe { Handle::new(&list) })
}

/// A list of objects that may be shared with the kernel. `List` is implemented
/// for system call types such as `Subscribe`, as well as (potentially-nested)
/// tuples of such types.
pub trait List: Default {}

#[cfg(test)]
mod tests;
