Subscribe Syscall API Design
============================

This document gives a high-level overview of a design for the interface to Tock
2.0's Subscribe system call. It is designed to be approachable by an audience
that is not familiar with Tock.

## Tock 2.0 Subscribe system call overview

The Subscribe system call is used by userspace processes to register callbacks
with the Tock kernel, and for the purposes of this document can be represented
by the following interface:

```rust
trait Callback {
    fn callback(&self, args: [u32; 3]);
}

struct InvalidIdError;

// Safety requirement: The process must call kernel_unsubscribe for this ID
// before the `callback` argument becomes invalid.
unsafe fn kernel_subscribe<CB: Callback>(callback: &CB, id: u32)
    -> Result<(), InvalidIdError>;

fn kernel_unsubscribe(id: u32) -> Result<(), InvalidIdError>;
```

Callbacks are not preemptive: they are only invoked when the process requests
they be invoked (via a separate system call), removing most (all?) thread-safety
concerns from the design.

Each ID represents a particular event the process can subscribe to, and not all
IDs are valid. If an ID is valid at one moment in time, it will remain valid
indefinitely.

Each call to `kernel_subscribe` overwrites the callback with the given ID,
replacing whatever callback was previously registered. Calling
`kernel_unsubscribe` removes the callback for the given ID, causing the kernel
to drop those events.

## Subscribe API design

`kernel_subscribe` from the previous section is `unsafe`. This section describes
an API intended to allow safe code to use the Subscribe system call.

This API is designed to support `Callback`s allocated on the stack, which have a
non-`'static` lifetime. The key difficulty in designing this API is making sure
that subscriptions are cleaned up before their `Callback` is deallocated. A
secondary goal is keeping the implementation lightweight: Tock is an embedded
OS, and a typical Tock process will likely make many Subscribe calls.

The first element of the API is `Unsubscriber`. `Unsubscriber` is a type that
removes a subscription when it is dropped:

```rust
#[derive(Default)]
struct Unsubscriber<'callback, const ID: u32> {
    // Makes Unsubscriber invariant with respect to 'callback.
    _phantom: PhantomData<Cell<&'callback ()>>,
}

impl<'callback, const ID: u32> Drop for Unsubscriber<'callback, ID> {
    fn drop(&mut self) {
        unsafe {
            // Guaranteed to succeed if ID is valid. If ID is invalid, then we
            // know there isn't a subscription to overwrite. Therefore we don't
            // need to handle errors here.
            let _ = kernel_unsubscribe(ID);
        }
    }
}
```

However, the existence of an `Unsubscriber` does *not* guarantee the `Drop`
implementation will be called (see the [safe `mem::forget`
RFC](https://rust-lang.github.io/rfcs/1066-safe-mem-forget.html)). Therefore, we
cannot soundly add a safe `subscribe` method to `Unsubscriber`.

Instead, we introduce a new handle type, whose existence guarantees that the
`Unsubscriber` will be dropped. The technique it uses to make the drop guarantee
is inspired by [`Pin`](https://doc.rust-lang.org/core/pin/index.html): make
`new` unsafe and tell the caller to guarantee it will be dropped correctly:

```rust
// Type invariant: a SubscribeHandle's existence guarantees that an Unsubscriber
// will clear the subscription identified by ID before the 'callback lifetime
// ends.
#[derive(Clone, Copy)]
struct SubscribeHandle<'callback, const ID: u32> {
    // Makes SubscribeHandle invariant with respect to 'callback, and ties its
    // lifetime to the Unsubscriber.
    _phantom: PhantomData<&'callback Unsubscriber<'callback, ID>>,
}

impl<'callback, const ID: u32> SubscribeHandle<'callback, ID> {
    // Safety: The caller is responsible for guaranteeing that `Drop::drop` will
    // be invoked before the 'callback lifetime ends.
    pub unsafe fn new(unsubscriber: &Unsubscriber<'callback, ID>) -> Self {
        SubscribeHandle {
            _phantom: PhantomData,
        }
    }
}
```

`SubscribeHandle`'s invariant allows us to add a safe `subscribe` method to
`SubscribeHandle`:

```rust
impl<'callback, const ID: u32> SubscribeHandle<'callback, ID> {
    fn subscribe<CB: Callback>(self, callback: &'callback CB)
        -> Result<(), InvalidIdError>
    {
        unsafe {
            // Safety: kernel_subscribe requires that unsubscribe is called
            // before callback becomes invalid, which happens at the end of the
            // 'callback lifetime. That is guaranteed by the type invariant of
            // SubscribeHandle.
            kernel_subscribe(callback, ID)
        }
    }
}
```

We can use the same trick as
[`pin-utils::pin_mut`](https://docs.rs/pin-utils/0.1.0/pin_utils/macro.pin_mut.html)
to create a safe macro that creates a `SubscribeHandle`:

```rust
macro_rules! subscribe_handle {
    ($($name:ident),* $(,)?) => { $(
        let $name = Unsubscriber { _phantom: PhantomData };
        // Shadow the variable to prevent the caller from forgetting the
        // Unsubscriber.
        let $name = unsafe { SubscribeHandle::new($name) };
    )* }
}
```

## Example usage

Here is an example of how the Subscribe API would be used:

```rust
struct App;

impl Callback for App {
    fn callback(&self, args: [u32; 3]) {
        // Insert callback logic here.
    }
}

impl App {
    fn run<'self>(&'self self, handle: SubscribeHandle<'self, 1>) {
        handle.subscribe(self);
    }
}

fn main() {
    let app = App;
    subscribe_handle!(handle);
    app.run(handle);
    // Insert main loop here.
}
```
