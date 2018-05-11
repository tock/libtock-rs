# [DRAFT] Design

## Driver Access

### The Problem

The current API provides driver access to any component at any point in time. This design is problematic as we can, for example, retrieve an instance of `GpioRead` and `GpioWrite` for the same GPIO port at the same time. Using Rust's ownership and borrowing concept we should be able to prevent such errors at compile time. But before we can bring `libtock` to an idiot proof level we should agree on some API design first.

### Proposed Solution

#### Global Drivers object

There is a global Drivers object that can exist only once at a time. It must not be copyable or cloneable.

```rust
pub fn drivers() -> Option<Drivers> {
    // Return Some(...) when this method is called for the first time and None afterwards
}
```

The Drivers object provides access to all drivers.

```rust
pub struct Drivers {
    pub foo: FooDriver,
    pub bar: BarDriver,
    // ... etc.
}
```

The user can pick the drivers he needs. Ownership of the drivers is retrieved via destructuring.

```rust
let Drivers { buttons, leds, .. } = tock::drivers().unwrap();
```

[OPTIONAL] When the Drivers object is destroyed, it could be made available in `tock::drivers()` again. However, this cannot work in combination with the destructuring mentioned above. Maybe, switching to `&mut`s in `Drivers` might do the trick.

```rust
impl Drop for Drivers {
    pub fn drop(&mut self) {
        // Make Drivers available for tock::drivers() again
    }
}
```

There should be no way to use a driver in two places at the same time. A first protective measure would be to disable the instantiation of drivers outside of `libtock`.

```rust
pub struct FooDriver {
    pub(crate) _private: () // Struct is zero-sized and can only be instantiated by libtock
}
```

A second measure would be to retain a mutable borrow as long as a driver is initialized and active. That way, before initializing a driver again, the mutable borrow must end first i.e. the driver must no longer be used.

```rust
impl FooDriver {
    pub fn init<'a>(&'a mut self) -> Foo<'a> { /* ... */ }
    pub fn init<'a>(&'a mut self) -> Option<Foo<'a>> { /* ... */ }
    pub fn init<'a>(&'a mut self) -> TockResult<Foo<'a>, FooInitializationError> { /* ... */ }
    pub fn init<'a, F: FnMut(...)>(&'a mut self, callback: F) -> Foo<'a> { /* ... */ }
}
```

In order to keep the mutable borrow active during the lifetime of the initialized driver, an artificial lifetime constraint (`&'a()`) has to be added.

```rust
struct Foo<'a> {
    _lifetime: &'a(),
}
```

#### Drivers API

Some drivers encapsulate access to multiple components of a kind. The individual components are accessed conveniently using iterators:

```rust
for gpio in gpios { /* ... */ }
```

```rust
struct Gpios<'a> {
    _lifetime: &'a(),
}

impl<'a, 'b> IntoIterator for &'b mut Buttons<'a> {
    type Item = ButtonHandle<'b>;
    type IntoIter = ButtonIter<'b>;
    /* ... */
}

impl<'a> Iterator for GpioIter<'a> {
    type Item = GpioHandle<'a>;
    /* ... */
}
```

Again, using Rust's ownership system we can make sure that components are enabled in a consistent state. Some examples:

```rust
impl<'a> ButtonHandle<'a> {
    pub fn enable(&'b mut self) -> TockResult<Button<'b>, ButtonActivationError> {
}

impl<'a> Button<'a> {
    pub fn read(&self) -> TockResult<ButtonState, ButtonAccessError> { /* ... */ }
}

impl<'a> GpioHandle<'a> {
    pub fn enable_read(&'b mut self) -> TockResult<GpioRead<'b>, GpioActivationError> {

    pub fn enable_write(&'b mut self) -> TockResult<GpioWrite<'b>, GpioActivationError> {
}

impl<'a> GpioRead<'a> {
    pub fn read(&self) -> TockResult<GpioState, GpioAccessError> { /* ... */ }
}

impl<'a> GpioWrite<'a> {
    pub fn write(&self, state: GpioState) -> TockReslut<(), GpioAccessError> { /* ... */ }
}
```

[TODO] Find a good name for components that are not activated and ready for use yet. Ideas:
- `FooHandle`
- `FooAccess`

#### Remaining Problems

[TODO] How can we prevent that two concurrent Tock processes access the same component?

## Subscriptions

### Callback Signature

Tock guarantees mutually exclusive execution of the main thread and the registered callbacks. Therefore, Rust callbacks can safeley consume `&mut self` and need not be neither `Send` nor `Sync`. In order to invoke a typed Rust callback from the untyped Tock ABI, we need a callback adapter function `call_rust`:

```rust
trait SubscribableCallback {
    fn call_rust(&mut self, arg0: usize, arg1: usize, arg2: usize);
}
```

where `arg{0,1,2}` are the first arguments passed to the Tock callback and `&mut self` is a reference to the Rust callback, obtained from the userdata part of the Tock callback.

### Lifetime of a callback

In general, Rust callbacks keep some internal state. This state must not be destroyed while the subscription is still active.

This problem can be solved by wrapping the callback into a `CallbackSubscription` type that unsubscribes the callback on `drop()`.

```rust
fn subscribe<CB: SubscribableCallback>(mut callback: CB) -> CallbackSubscription<CB> {
    // Create a Tock callback that calls the Rust callback via `call_rust`
    // Subscribe the Tock callback
}

impl<CB: SubscribableCallback> Drop for CallbackSubscription<CB> {
    // Unsubscribe the Tock callback
}
```

## Allow

### Sharing memory

Userspace processes can use 'allow' to allow drivers accessing
memory. Processes can only share byte arrays of fixed lengths with the drivers.

```rust
pub struct SharedMemory<'a> {
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: &'a mut [u8],
}

impl<'a> SharedMemory<'a> {
    pub fn read_bytes(&self, destination: &mut [u8]) {
        safe_copy(self.buffer_to_share, destination);
    }

    pub fn write_bytes(&mut self, source: &[u8]) {
        safe_copy(source, self.buffer_to_share);
    }
}
```


This grants the driver write access to the byte array. This implies that
the write access has to be forbidden, once the shareable memory goes out of scope.
Analogously as in the callback case this is achieved by returning a 'SharedMemory' object
and implementing 'Drop' for it.
```rust
impl<'a> Drop for SharedMemory<'a> {

}
```

### Using shared memory

The common sceneario for using shared memory is as follows:
 - share a buffer with the driver
 - register a callback reading communicating with the driver via the shared memory

The buffers can only be written/read using the provided methods `write_bytes`/`read_bytes`.

# Error Handling
Libtock has two kinds of errors: Internal errors and Syscall errors:
```rust
enum TockError {
    InternalError,
    SyscallError,
}
```
Internal errors can be of the following kind:
```rust
enum InternalError {
    OutOfMemory,
    BleParseError,
}
```
Syscall errors are caused by syscalls having a result `<0`:
```rust
struct SyscallError {
    syscall: Syscall,
    driver: usize,
    arg1: isize,
    arg2: isize,
    return_value: isize,
}
```
where syscall is one of the following:
```rust
enum Syscall {
    Allow,
    Subscribe,
    Memop,
    Command,
}
```
Errors propaged by libtock can be evaluated in the business logic of an application for example
by implementing a custom error type and implementing the `From` trait for it.