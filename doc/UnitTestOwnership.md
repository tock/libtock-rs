Unit Test Ownership Design
==========================

**TODO: This document describes a not-yet-implemented design for
`libtock_unittest`. This TODO will be removed when `libtock_unittest` is
refactored to use the new design.**

This document examines the component-level call graph for unit tests, including
the code under test, fake kernel, and fake syscall drivers. The call graph
contains both shared references and circular references, so representing it via
Rust's ownership system is nontrivial. This document derives an ownership
strategy for the fake kernel and fake syscall drivers.

## The Call Graph

For our current purposes, we are looking at the interactions between the
following components:

1. **Unit test:** The `#[test]` function, which sets up the test environment and
   drives the test execution.
2. **Code under test:** The code which we are testing, which should not depend
   on `libtock_unittest` (as this code should run on Tock as well as the host
   system).
3. **`fake::Kernel`:** A single instance of `fake::Kernel` that handles system
   calls.
4. **`fake::SyscallDriver`:** 0 or more `fake::SyscallDriver` instances that
   implement system call interfaces.

The function calls we anticipate are as follows:

1. **Unit test -> Code under test:** The test case will instantiate components
   to test and call into them to execute the test.
2. **Unit test -> `fake::Kernel`:** The test case will create a fake kernel and
   call into it to configure it, including registering `fake::SyscallDriver`s
   with it.
3. **Unit test -> `fake::SyscallDriver`:** The test case may talk to fake
   syscall drivers to configure them or inspect their state (e.g. to verify
   whether the code under test performed particular actions).
4. **Code under test -> `fake::Kernel`:** The code under test will execute
   syscalls, which call into the fake kernel.
5. **`fake::Kernel` -> Code under test:** When the code under test invokes the
   Yield system call, the fake kernel may call back into the code under test to
   execute an upcall.
6. **`fake::Kernel` -> `fake::SyscallDriver`:** The fake kernel will call into
   fake syscall drivers as part of registering them, and to invoke Command
   system calls (and perhaps other system calls as well).
7. **`fake::SyscallDriver` -> `fake::Kernel`:** The fake syscall drivers will
   call into the fake kernel to queue upcalls and access Allow buffers.

The **`fake::Kernel` -> Code under test** calls always occur via function
pointer, with the function pointer passed via the Subscribe system call, so we
do not need to represent them directly in Rust's ownership system. Therefore I
will represent them via a dotted line.

I'll use the term "share access" to refer to fake syscall drivers queueing
upcalls and accessing Allow buffers, as `libtock_platform` uses that terminology
for that same functionality.

This gives the following call graph:

```
      +----------------------------------------+
      |             Unit test case             |
      +----------------------------------------+
       |                    |                 |
       V                    V                 V
+-------+  RawSyscalls  +--------+  Command  +=========+
| Code  |-------------->| Fake   |---------->| Fake    |
| under |               | kernel |           | syscall |
| test  |< - - - - - - -|        |<----------| drivers |
+-------+  Upcall       +--------+   Share   +=========+
           invocation                access
```

There are a few things to note here:

1. The `fake::Kernel` is shared between the unit test case and the code under
   test.
2. The `fake::SyscallDriver` instances are shared between the unit test case and
   the `fake::Kernel`.
3. There is a circular dependency between the `fake::Kernel` and the
   `fake::SyscallDriver`s.

Note that we can ignore the shared access to the code under test, because that
is handled by `libtock_platform`'s syscall API design.

## Handling the `fake::Kernel` <-> `fake::SyscallDriver` circular dependency

We cannot do both of the following:

1. Store `&dyn fake::SyscallDriver` references in the `fake::Kernel`.
2. Store `&fake::Kernel` references in the `fake::SyscallDriver`s.

because those references types require lifetime parameters, and drop check will
not pass (both `fake::Kernel` and the `fake::SyscallDriver`s use dynamic memory
allocation). Therefore, we need to give up on one of the above.

Fortunately, number 2 (storing references to the kernel inside the fake syscall
drivers) isn't quite what we want anyway. Fake syscall drivers should have
access to share data associated with their driver number, but not other drivers.
So it makes more sense to give them a handle type, which we can call
`DriverShareRef`, which only gives them access to their own shares. To avoid the
drop check issues, `DriverShareRef` cannot have a lifetime parameter, so the
share data itself needs to be pulled out into a separate object:

```
      +----------------------------------------+
      |             Unit test case             |
      +----------------------------------------+
       |                    |                 |
       V                    V                 V
+-------+  RawSyscalls  +--------+  Command  +=========+
| Code  |-------------->| Fake   |---------->| Fake    |
| under |               | kernel |           | syscall |
| test  |< - - - - - - -|        |           | drivers |
+-------+  Upcall       +--------+           +=========+
           invocation           |             |
                                | Rc<>        | DriverShareRef
                                V             V
                               +---------------+
                               | ShareData     |
                               +---------------+
```

`ShareData` would contain data common to all `fake::SyscallDriver`s (such as the
upcall queue).

`DriverShareRef`'s API will allow the `fake::SyscallDriver`s to read Allow
buffers shared with them as well as queue upcalls. As such, it will need to
contain the following data:

```rust
struct DriverShareRef {
    driver_num: u32,
    share_data: Rc<ShareData>,
}
```

To give the `DriverShareRef` to the `fake::SyscallDriver`s, we need to add a
registration function to `fake::SyscallDriver`:

```rust
trait SyscallDriver {
    /* ... */

    fn register(&self, share_ref: DriverShareRef);
}
```

## Remaining details

With the above design for `fake::Kernel` and `fake::SyscallDriver`, the unit
test case can directly own both the `fake::Kernel` and the
`fake::SyscallDriver`s. The `fake::Kernel` can hold `&dyn fake::SyscallDriver`s.

If we implement `RawSyscalls` on `&fake::Kernel`, then we can avoid using
thread-local storage.

This will require the `fake::Kernel` and `fake::SyscallDriver`s to use interior
mutability.
