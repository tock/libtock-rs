`libtock_platform` Design Story
===============================

`libtock_platform` is a crate that will contain core abstractions that will be
used by `libtock_core`'s drivers. For example, it will contain abstractions for
asynchronous APIs that are lighter weight (in terms of code size and RAM usage)
than Rust's futures.

This document serves as a justification for, and explanation of, the high-level
design of `libtock_platform`. Instead of describing the various components and
how they interact, it starts with a hello world application and extracts some of
the functionality out into a reusable library. As we do so, we see that
`libtock_core`'s [design considerations](DesignConsiderations.md) lead
incrementally to a design for `libtock_platform`.

In order to keep things understandable, this document makes several
simplifications. Error handling logic is omitted, although we still structure
the code so as to allow it. We use `unsafe` in places where we should instead
have an additional, safe, abstraction. We use a simplified form of the Tock 2.0
system calls, which are currently undergoing design revisions.

## Hello World

We start with the following example app. In order to show how `libtock_platform`
will handle asynchronous callbacks, it is written in an asynchronous manner.

```rust
#![no_std]

static GREETING: [u8; 7] = *b"Hello, ";
static NOUN: [u8; 7] = *b"World!\n";
static mut DONE: bool = false;

// Driver, command, allow, and subscription numbers.
const DRIVER: usize = 1;
const WRITE_DONE: usize = 1;
const WRITE_BUFFER: usize = 1;
const START_WRITE: usize = 1;

fn main() {
    libtock_runtime::subscribe(DRIVER, WRITE_DONE, write_complete, 0);
    libtock_runtime::const_allow(DRIVER, WRITE_BUFFER, &GREETING);
    libtock_runtime::command(DRIVER, START_WRITE, GREETING.len(), 0);
    loop {
        libtock_runtime::yieldk();
    }
}

extern "C" fn write_complete(bytes: usize, _: usize, _: usize, _data: usize) {
    // Detect if this write completion is a result of writing NOUN (the
    // second write). If so, return immediately to avoid an infinite loop.
    unsafe {
        if DONE { return; }
        DONE = true;
    }
    // At this point, we just finished writing GREETING and need to write NOUN.
    libtock_runtime::const_allow(DRIVER, WRITE_BUFFER, &NOUN);
    libtock_runtime::command(DRIVER, START_WRITE, NOUN.len(), 0);
}
```

Strictly speaking, in this app, `DONE` could be tracked by passing it as the
`data` argument to `subscribe`. However, most applications will need to track
more than 32 bits of persistent state -- and use more than one callback -- so
they will need to manage persistent state themselves. To make this app
representative of most apps, we keep the state in userspace memory.

This example assumes that the `libtock_runtime` crate exposes system call
implementations that look like the following (these are a simplification of the
Tock 2.0 system calls):

```rust
pub fn subscribe(driver_num: usize,
                 subscription_num: usize,
                 callback: extern "C" fn(usize, usize, usize, usize),
                 data: usize);

pub fn const_allow(driver_num: usize, buffer_num: usize, buffer: &'static [u8]);
pub fn command(driver_num: usize, command_num: usize, arg1: usize, arg2: usize);
pub fn yieldk();
```

What do we already get "right" in this example? Except for the two intentional
inefficiencies explained above (performing the write in two steps rather than 1
and maintaining the `DONE` state in process memory), it is very efficient. It
has only 3 global variables: `GREETING`, `NOUN`, and `DONE`. It has almost zero
bookkeeping overhead; it fairly directly makes 8 system calls (`subscribe`,
`const_allow`, `command`, `yieldk`, `const_allow`, `command`, `yieldk`,
`yieldk`).

What do we want to improve? It calls system calls directly from application code
-- there should be a reusable `console` library that implements the system calls
instead! The application shouldn't need to know what command number starts a
write, it should just tell the console driver to do that for it.

## Adding a Console Library

Let's start taking some of the console-specific parts and moving them into a new
crate. The first system call the app makes is to `subscribe` to the write done
callback, so let's add a function to a new `libtock_console` crate that sets a
write callback using `subscribe`:

```rust
#![no_std]

const DRIVER: usize = 1;
const WRITE_DONE: usize = 1;

fn set_write_callback(callback: fn(bytes: usize, data: usize), data: usize) {
    libtock_runtime::subscribe(DRIVER, WRITE_DONE, write_complete, data);
    // We need to store `callback` somewhere -- where to do so?
}

extern "C" fn write_complete(bytes: usize, _: usize, _: usize, data: usize) {
    // Hmm, how do we access the callback?
}
```

You may notice that the code is not quite complete: `set_write_callback` takes a
callback but doesn't store it anywhere. We don't want to store it in a global
variable because doing so would not be zero-cost: the original app didn't need
to store a function pointer, and we want to do this refactoring without bloating
the app. We could pass it through `data`, but what if the client code needs to
use `data` itself? That pattern isn't extensible: if there is another
asynchronous layer about the console (e.g. a virtualization system), it won't
have access to `data` to pass its callbacks through.

Instead, we can pass the callback through the type system. We need a new trait
to represent the callback. This trait won't be specific to `libtock_console`,
and we'll later use it in unit tests -- which run on Linux, not Tock, so we'll
put it in the `libtock_platform` crate:

```rust
pub trait FreeCallback<AsyncResponse> {
    fn call(response: AsyncResponse);
}
```

We call this `FreeCallback` because it represents a free function as opposed to
a method. (This forshadows `MethodCallback`, which we will need later)

The reason why we made this a shared generic trait rather than adding a
`libtock_console::Client` trait as the Tock kernel does will be apparent later.

Using this trait, we can now write `libtock_console::set_write_callback`:

```rust
#![no_std]

use libtock_platform::FreeCallback;

const DRIVER: usize = 1;
const WRITE_DONE: usize = 1;

pub struct WriteCompleted {
    pub bytes: usize,
    pub data: usize,
}

pub fn set_write_callback<Callback: FreeCallback<WriteCompleted>>(data: usize) {
    libtock_runtime::subscribe(DRIVER, WRITE_DONE, write_complete::<Callback>, data);
}

extern "C" fn write_complete<Callback: FreeCallback<WriteCompleted>>(
    bytes: usize, _: usize, _: usize, data: usize)
{
    Callback::call(WriteCompleted { bytes, data });
}
```

To finish `libtock_console`, we also need to add `set_write_buffer` (which calls
`allow`) and `start_write` (which calls `command`), which are much simpler:

```rust
const WRITE_BUFFER: usize = 1;
const START_WRITE: usize = 1;

pub fn set_write_buffer(buffer: &'static [u8]) {
    libtock_runtime::const_allow(DRIVER, WRITE_BUFFER, buffer);
}

pub fn start_write(bytes: usize) {
    libtock_runtime::command(DRIVER, START_WRITE, bytes, 0);
}
```

We can then make use of `libtock_console` in our app as follows:

```rust
#![no_std]

static GREETING: [u8; 7] = *b"Hello, ";
static NOUN: [u8; 7] = *b"World!\n";
static mut DONE: bool = false;

fn main() {
    libtock_console::set_write_callback::<App>(0);
    libtock_console::set_write_buffer(&GREETING);
    libtock_console::start_write(GREETING.len());
    loop {
        libtock_runtime::yieldk();
    }
}

struct App;

impl libtock_platform::FreeCallback<libtock_console::WriteCompleted> for App {
    fn call(_response: WriteCompleted) {
        unsafe {
            if DONE { return; }
            DONE = true;
        }
        libtock_console::set_write_buffer(&NOUN);
        libtock_console::start_write(NOUN.len());
    }
}
```

Now we have a reusable console library! However, we still don't have any unit
tests.

## Adding a Fake Kernel

A good unit test for the console driver would test not only the driver's
operation with successful system calls but also also test the driver's
error-handling logic. That is difficult to do if we test using a real Tock
kernel in an emulator -- the real kernel has the goal of avoiding system call
errors! Instead of using a real Tock kernel, driver unit tests should use a
"fake kernel" that simulates the kernel's functionality while allowing errors to
be injected.

To keep this document reasonably short and understandable, we have omitted error
handling, but we can still structure our unit tests in a manner that would allow
a test to inject errors when error handling logic is added.

To allow the console driver to work with both a real kernel and a fake kernel,
we add a `Syscalls` trait to `libtock_platform`. When compiled into a Tock
process binary, `Syscalls` will be implemented by a zero-sized type, which
avoids wasting non-volatile storage space or RAM area. To avoid passing around
references to a `Syscalls` implementation, we can pass the `Syscalls` by value
rather than by reference (i.e. take `self` rather than `&self`). For practical
use, this requires the `Syscalls` implementations to be `Copy`.

```rust
pub trait Syscalls: Copy {
    fn subscribe(self, driver: usize, minor: usize,
                 callback: extern "C" fn(usize, usize, usize, usize),
                 data: usize);

    fn const_allow(self, major: usize, minor: usize, buffer: &'static [u8]);
    fn command(self, major: usize, miner: usize, arg1: usize, arg2: usize);
    fn yieldk(self);
}
```

We then implement `Syscalls` using a real Tock kernel in `libtock_runtime`:

```rust
#[derive(Clone, Copy)]
pub struct TockSyscalls;

impl libtock_platform::Syscalls for TockSyscalls {
    /* Omitted implementation details */
}
```

We adapt `libtock_console` to use an app-provided `Syscalls` implementation
rather than directly calling into `libtock_runtime`:

```rust
pub fn set_write_callback<S: Syscalls, Callback: FreeCallback<WriteCompleted>>(
    syscalls: S, data: usize)
{
    syscalls.subscribe(1, 1, write_complete::<Callback>, data);
}

extern "C" fn write_complete<Callback: FreeCallback<WriteCompleted>>(
    bytes: usize, _: usize, _: usize, data: usize)
{
    Callback::call(WriteCompleted { bytes, data });
}

pub fn set_write_buffer<S: Syscalls>(syscalls: S, buffer: &'static [u8]) {
    syscalls.const_allow(1, 1, buffer);
}

pub fn start_write<S: Syscalls>(syscalls: S, bytes: usize) {
    syscalls.command(1, 1, bytes, 0);
}
```

We'll create a new crate, `libtock_unittest`, which contains test utilities such
as the fake Tock kernel. The fake kernel, unlike
`libtock_runtime::TockSyscalls`, needs to maintain state, so it cannot be a
zero-sized type. Instead of implementing `Syscalls` on the fake kernel directly,
we implement it on a shared reference:

```rust
type RawCallback = extern "C" fn(usize, usize, usize, usize);

pub struct FakeSyscalls {
    callback_pending: core::cell::Cell<Option<usize>>,
    output: core::cell::Cell<Vec<u8>>,
    write_buffer: core::cell::Cell<Option<&'static [u8]>>,
    write_callback: core::cell::Cell<Option<RawCallback>>,
    write_data: core::cell::Cell<usize>,
}

impl FakeSyscalls {
    pub fn new() -> Self {
        FakeSyscalls {
            callback_pending: Cell::new(None),
            output: Cell::new(Vec::new()),
            write_buffer: Cell::new(None),
            write_callback: Cell::new(None),
            write_data: Cell::new(0),
        }
    }

    pub fn read_buffer(&self) -> &'static [u8] {
        self.write_buffer.take().unwrap_or(&[])
    }
}

impl libtock_platform::Syscalls for &FakeSyscalls {
    fn subscribe(self, driver: usize, minor: usize, callback: RawCallback, data: usize) {
        if driver == 1 && minor == 1 {
            self.write_callback.set(Some(callback));
            self.write_data.set(data);
        }
    }

    fn const_allow(self, major: usize, minor: usize, buffer: &'static [u8]) {
        if major == 1 && minor == 1 {
            self.write_buffer.set(Some(buffer));
        }
    }

    fn command(self, major: usize, minor: usize, arg1: usize, _arg2: usize) {
        if major != 1 || minor != 1 { return; }
        if let Some(buffer) = self.write_buffer.get() {
            let mut output = self.output.take();
            let bytes = core::cmp::min(arg1, buffer.len());
            output.extend_from_slice(&buffer[..bytes]);
            self.output.set(output);
            self.callback_pending.set(Some(bytes));
        }
    }

    fn yieldk(self) {
        let bytes = match self.callback_pending.take() {
            Some(bytes) => bytes,
            None => return,
        };
        if let Some(callback) = self.write_callback.get() {
            callback(bytes, 0, 0, self.write_data.get());
        }
    }
}
```

## Adding a Synchronous Adapter

We're still not ready to add unit tests to `libtock_console` yet!
`libtock_console` is asynchronous, which is difficult to work with in a unit
test. `libtock_core` should provide synchronous APIs for apps that do not wish
to be fully asynchronous, so lets go ahead and implement a synchronous API. To
avoid re-implementing synchronous APIs for every driver, let's make it work
generically with all `libtock_core` drivers. This is where we benefit from
making `FreeCallback` a generic trait rather than having a
`libtock_console::Client` trait.

The synchronous adapter will need to store a copy of an `AsyncResponse`, so its
callback cannot be a free function (it needs access to `self`). Therefore, we
add the `MethodCallback` trait to `libtock_platform`:

```rust
pub trait MethodCallback<AsyncResponse> {
    fn call(&self, response: AsyncResponse);
}
```

Using `MethodCallback`, we can now write `SyncAdapter`. We add `SyncAdapter` to
a new crate, `libtock_sync`, as not all Tock apps will want it:

```rust
use libtock_platform::MethodCallback;

pub struct SyncAdapter<AsyncResponse, Syscalls> {
    response: core::cell::Cell<Option<AsyncResponse>>,
    syscalls: Syscalls,
}

impl<AsyncResponse, Syscalls> SyncAdapter<AsyncResponse, Syscalls> {
    pub const fn new(syscalls: Syscalls) -> SyncAdapter<AsyncResponse, Syscalls> {
        SyncAdapter { response: core::cell::Cell::new(None), syscalls }
    }
}

impl<AsyncResponse, Syscalls: libtock_platform::Syscalls> SyncAdapter<AsyncResponse, Syscalls> {
    pub fn wait(&self) -> AsyncResponse {
        loop {
            match self.response.take() {
                Some(response) => return response,
                None => self.syscalls.yieldk(),
            }
        }
    }
}

impl<AsyncResponse, Syscalls: libtock_platform::Syscalls>
MethodCallback<AsyncResponse> for SyncAdapter<AsyncResponse, Syscalls> {
    fn call(&self, response: AsyncResponse) {
        self.response.set(Some(response));
    }
}
```

## Adding a Unit Test

Before we write the test itself, we should add one more utility to
`libtock_unittest`. That utility is the `test_component!` macro, which creates a
thread-local instance of a type and provides `FreeCallback` implementations for
every `MethodCallback` implementation the type has:

```rust
#[macro_export]
macro_rules! test_component {
    [$link:ident, $name:ident: $comp:ty = $init:expr] => {
        let $name = std::boxed::Box::leak(std::boxed::Box::new($init)) as &$comp;
        std::thread_local!(static GLOBAL: std::cell::Cell<Option<&'static $comp>>
                           = const {std::cell::Cell::new(None)})
        GLOBAL.with(|g| g.set(Some($name)));
        struct $link;
        impl<T> libtock_platform::FreeCallback<T> for $link
        where $comp: libtock_platform::MethodCallback<T> {
            fn call(response: T) {
                GLOBAL.with(|g| g.get().unwrap()).call(response);
            }
        }
    };
}
```

We can finally add a unit test to `libtock_console`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn write() {
        extern crate std;

        use libtock_platform::MethodCallback;
        use libtock_sync::SyncAdapter;
        use libtock_unittest::FakeSyscalls;
        use std::boxed::Box;
        use std::thread_local;
        use super::{set_write_buffer, set_write_callback, start_write, WriteCompleted};

        let syscalls: &_ = Box::leak(Box::new(FakeSyscalls::new()));
        libtock_unittest::test_component![SyncAdapterLink, sync_adapter: SyncAdapter<WriteCompleted, &'static FakeSyscalls>
                                          = SyncAdapter::new(syscalls)];

        set_write_callback::<_, SyncAdapterLink>(syscalls, 1234);
        set_write_buffer(syscalls, b"Hello");
        start_write(syscalls, 5);
        let response = sync_adapter.wait();
        assert_eq!(response.bytes, 5);
        assert_eq!(response.data, 1234);
        assert_eq!(syscalls.read_buffer(), b"Hello");
    }
}
```

## Adding `static_component!`

We added `libtock_unittest::test_component!` to make it easy to set up
components in unit tests, but we have no equivalent for apps. Our app still uses
`unsafe` to access its `DONE` variable. Instead, lets hide that unsafety behind
a new macro. This macro is only sound in `libtock_runtime`'s single-threaded
environment, so we add it to `libtock_runtime` directly:

```rust
#[macro_export]
macro_rules! static_component {
    [$link:ident, $name:ident: $comp:ty = $init:expr] => {
        static mut COMPONENT: $comp = $init;
        struct $link;
        impl<T> libtock_platform::FreeCallback<T> for $link
        where $comp: libtock_platform::MethodCallback<T> {
            fn call(response: T) {
                unsafe { &COMPONENT }.call(response);
            }
        }
    };
}
```

We can now use `static_component!` in our Hello World app to instantiate the
`App` struct:

```rust
#![no_std]

static GREETING: [u8; 7] = *b"Hello, ";
static NOUN: [u8; 7] = *b"World!\n";

fn main() {
    libtock_console::set_write_callback::<_, AppLink>(0);
    libtock_console::set_write_buffer(&GREETING);
    libtock_console::start_write(GREETING.len());
    loop {
        libtock_runtime::yieldk();
    }
}

struct App {
    done: core::cell::Cell<bool>
}

impl App {
    pub const fn new() -> App {
        App {
            done: core::cell::Cell::new(false)
        }
    }
}

impl MethodCallback<WriteCompleted> for App {
    fn call(&self, _response: WriteCompleted) {
        if self.done.get() { return; }
        self.done.set(true);
        set_write_buffer(TockSyscalls, &NOUN );
        start_write(TockSyscalls, NOUN.len() );
    }
}

libtock_runtime::static_component![AppLink, APP: App = App::new()];
```

Now our app has no more unsafe!

## Recap

We wrote a Hello World application that uses Tock's console system calls and
asynchronous callbacks. We then extracted the console system call interface into
a reusable library, creating the `FreeCallback` trait along the way.

In order to provide unit tests for the console library, we needed to create
several new abstractions. We created `Syscalls` so that we can direct tho
console driver's system calls to a fake kernel. We created the
`libtock_unittest` crate which contains the fake kernel as well as a
`test_component!` helper macro. We created `SyncAdapter` so that the unit test
can be written in a synchronous manner -- although `SyncAdapter` is not
testing-specific! We created `MethodCallback` because `FreeCallback` is not a
powerful-enough abstraction on its own for `SyncAdapter`.

We ended up with six Rust crates:

1. `libtock_console`, which contains the console driver logic and unit test
   code.
2. `libtock_platform`, which provides abstractions that can be shared with other
   drivers.
3. `libtock_runtime`, which contains non-portable system call implementations.
4. `libtock_sync`, which provides a synchronous interface using
   `libtock_platform`'s traits.
5. `libtock_unittest`, which provides a fake Tock kernel and other utilities.
6. The hello world app itself.
