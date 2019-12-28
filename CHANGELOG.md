# Releases

## 0.2.0 (WIP)

- Many functions are asynchronous
  - To make create an `async` main function you can use the attribute `#[libtock::main]`
  - To retrieve the value of an asynchronous `value`, use `value.await`
  - This is only possible within an `async fn`, so either
    - Make the caller `fn` of `.await` an `async fn`
    - Not recommended: Use `core::executor::block_on(value)` to retrieve the `value`
- `syscalls::yieldk_for` is no longer available
  - Yielding manually is discouraged as it conflicts with Rust's safety guarantees. If you need to wait for a condition, use `futures::wait_until` and `.await`.
- `syscalls::yieldk` has become `unsafe` for the same reason
- Commands are no longer `unsafe`
- The low-level syscalls have been moved to `syscalls::raw`
  - `syscalls::subscribe_ptr` becomes `syscalls::raw::subscribe`
  - `syscalls::allow_ptr` becomes `syscalls::raw::allow`
- Targets without support for atomics can be built
- Most API functions, including `main()`, return a `Result<T, TockError>`
- The timer now supports to be used simultaneously
- all drivers can exclusively be retrieved by `retrieve_hardware` which returns a `Hardware`-singleton. Drivers can be shared between different tasks only if it is safe to do so.

## a8bb4fa9be504517d5533511fd8e607ea61f1750 (0.1.0)

- First and highly experimental `libtock-rs` API
