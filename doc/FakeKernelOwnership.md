`libtock_unittest::fake` Ownership Design
=========================================

In process binaries, the majority of `libtock-rs` components are in `static`
variables and have `'static` lifetime. As a result, the ownership and lifetime
story for components (such as syscall driver interfaces) is very simple, and
`Drop` implementations are generally unnecessary.

Unit tests are a more complex story. Each test case creates a fake kernel, which
it must interact with directly (to add drivers, use the expected syscall queue,
and check the syscall log). The test case creates fake drivers, which it also
interacts with. These drivers are shared with the fake kernel, because the fake
kernel has to route system calls (such as Command) to the fake drivers. The fake
drivers need to call into the kernel in order to queue upcalls. Making
everything more complex, system calls can be reentrant: a Yield call can run a
callback that calls other system calls. This results in a dependency web where
many different pieces all need to talk to each other, which makes ownership
complex.

One important question is "how do we clean up when a unit test finishes?". We
could simply leak all the objects, which gives them all `'static` lifetimes, but
that has two drawbacks:

1. It leaks memory, which wastes resources and reduces the effectiveness of
   leak-checking tools such as Miri's leak checker.
2. Having an unclear ownership story for the objects in the unit test makes the
   code un-idiomatic and harder to understand.

To enable unit test cleanup and give tests a sensible ownership graph,
`libtock_unittest` contains some non-obvious design decisions, which this
document explains.

## Why can't everything be a local variable?

One inclination you might have is that the objects should all be in local
variables. To do this, they would need to be parameterized on a lifetime. In
pseudo-Rust:

```rust
trait libtock_platform::Syscalls<'a> { ... }

struct libtock_console::Console<'a, S: Syscalls> { ... }

trait libtock_unittest::fake::Driver<'a> { ... }
struct libtock_unittest::fake::Kernel<'a> { ... }
```

You can get remarkably far with this design. None of these types need to
implement `Drop`, so the drop check doesn't get involved, and they can all have
the same lifetime. Instead of having free functions in `RawSyscalls` and
`Syscalls`, you can have those traits require `Copy` and take `self` by value.
That allows them to be implemented on `&fake::Kernel`, while remaining
zero-sized types in process binaries.

You don't encounter real trouble until you try to implement
`async_traits::Locator` in a unit test (which is done by the expansion of
`test_component!`):

```rust
let mut console = Console::<fake::Kernel>::new(&kernel);
struct ConsoleLocator;
impl<'a> Locator<'a> for ConsoleLocator {
    type Target = Console::<'a, fake::Kernel>::new();

    fn locate() -> &'a Console::<'a, fake::Kernel> {
        // Uh oh, can't access the console local variable here.
    }
}
```

You cannot make `Locator` point at a non-`'static` object! By design, `Locator`
is intended to convey the location of a particular object via the type system,
without passing around an instance of the `Locator`. In Rust, that is
fundamentally incompatible with returning a reference to an object on the stack.

Sadly, to implement `Locator` correctly, we need to put some references into
either thread-local or global storage. Ultimately, this means that all the
lifetime parameters we've added must be `'static`, which makes them unnecessary.

## How do we own the fake kernel?

From the perspective of the test cases, it is nice to treat the fake kernel as a
local variable. The public API of `libtock_unittest` reflects that:
`fake::Kernel::new` creates a new `fake::Kernel` and returns it. This
`fake::Kernel` has useful methods assocated with the fake kernel, such as
`add_driver` and `take_syscall_log`.

However, in order to implement `RawSyscalls`, the fake kernel's data needs to be
accessible from a thread-local variable. An early design we attempted was to use
`std::rc::Rc` to own the `fake::Kernel` and have all the data in the
`fake::Kernel`, but that became confusing. The public API of `fake::Kernel`
looked like a standard object (as in OOP), but internally it was used as a
shared data store. Making the fake system call implementation and upcall
invocation work correctly in the face of reentrant calls became tricky.

To avoid this complexity, the fake kernel's data is not in `fake::Kernel`.
Instead, it has type `KernelData` and is stored directly in a thread-local
variable. `KernelData` has no behavior of its own: by design, it is only data
storage. `fake::Kernel`, `fake::Syscalls`, and `schedule_upcall` all access
`KernelData` as needed. This makes writing reentrant code in `fake::Syscalls`
and `fake::Kernel` much easier. The `KernelData` is created by
`fake::Kernel::new` and cleaned up when the `fake::Kernel` is dropped.

## How do we own `fake::Driver` instances?

There are two things that need to access the `fake::Driver` instances: the unit
test case (for configuration and inspection), and the fake kernel (for syscall
implementations). Therefore, the `fake::Driver` instances must be accessed
through shared references. This requires them to use interior mutability or be
accessed through `RefCell`. We prioritize the understandability of test code
over the implementation of `fake::Driver`, so `fake::Driver` uses interior
mutability.

We could have the fake kernel own the `fake::Drivers`, but doing so requires a
significant amount of `unsafe` code to satisfy the borrow checker (it really
doesn't like returning references into a struct that could be replaced at any
time). Instead, we use `Rc` to own the `fake::Drivers`. This results in the
following ownership graph:

```
+-----------+  +------------+
| Unit test |  | KernelData |
+-----------+  +------------+
         | Rc     | Rc
         V        V
      +--------------+
      | fake::Driver |
      +--------------+
```

A drawback of this design is that it is difficult for `fake::Driver` instances
to have references to other objects without adding cycles to this dependency
graph. Fortunately, it should be rare for `fake::Driver` instances to have
external dependencies.
