`libtock_unittest::fake` Ownership Design
=========================================

In process binaries, the majority of `libtock-rs` components are in `static`
variables and have `'static` lifetime. As a result, the ownership and lifetime
story for components (such as syscall driver interfaces) is very simple, and
`Drop` implementations are generally unnecessary.

Unit tests are a more complex story. Each test is a function that creates a
`fake::Kernel` which supports some number of `fake::Drivers`, as well as one or
more components to test. There are many interdependencies. Those components need
access to the `fake::Kernel`, but also pass callback pointers that get called by
the `fake::Kernel`. The `fake::Kernel` needs access to the `fake::Drivers` to
route system calls, and the `fake::Drivers` need access to the `fake::Kernel` to
queue upcalls. The test case needs access to the components, `fake::Kernel`, and
`fake::Drivers`, in order to invoke the functionality under test and inspect its
behavior.

This leaves open the question "how do we clean up when a unit test finishes?".
We could simply leak all the objects, which gives them all `'static` lifetimes,
but that has two drawbacks:

1. It leaks memory, which wastes resources and reduces the effectiveness of
   leak-checking tools such as Miri's leak checker.
2. Having an unclear ownership story for the objects in the unit test makes the
   code un-idiomatic and harder to understand.

To enable unit test cleanup and give tests a sensible ownership graph,
`fake::Kernel` and `fake::Driver` make use of `std::rc::Rc` for ownership. This
document describes the design and the reasons for the design.

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

## How do we own `fake::Kernel`?

Each unit test should create a single `fake::Kernel` for its use. In order to
implement `RawSyscalls`, that `fake::Kernel` must be accessible through either a
thread-local variable or a global variable. Multiple test cases can run in
parallel, so we want the `fake::Kernel` to be thread-specific. Therefore we
store the reference in a thread local variable.

In order to avoid leaking the `fake::Kernel`, we need the unit test to have some
sort of RAII handle that cleans it up when dropped. We could create our own
handle type (i.e. `FakeKernelHandle`), but that adds complexity to
`libtock_unittest` and makes it harder to understand for users. Instead, we use
`Rc`, a widely-known Rust type. The exact details are in
`userspace/src/kernel/thread_local.rs`, but the high-level overview is that a
`Rc<fake::Kernel>` is owned by the unit test case while a `Weak<fake::Kernel>`
is stored in a thread-local variable. This results in the following ownership
graph:

```
+-----------+  +--------------+
| Unit test |  | Thread local |
+-----------+  +--------------+
        | Rc      : Weak
        V         V
      +--------------+
      | fake::Kernel |
      +--------------+
```

### `fake::Kernel`: interior mutability, or `RefCell`?

`Rc` only gives us shared references to the `fake::Kernel`. This is fundamental
to the design: both the unit test case and the thread local variable have access
to `fake::Kernel`, so neither can be a unique reference. This means that we need
to use one of the following to mutate the data inside the `fake::Kernel`:

1. Interior mutability
2. Embed the `fake::Kernel` inside a `RefCell`.

The drawback of embedding the `fake::Kernel` inside a `RefCell` is the unit test
code will need to handle a `Rc<RefCell<fake::Kernel>>`, which is significantly
more verbose to work with than `Rc<fake::Kernel>`. There's a lot more unit test
code than `fake::Kernel` code, so we instead chose to use interior mutability.

`fake::Kernel` can use `RefCell` internally -- if it desires -- to implement its
interior mutability, so we don't lose much by going with this approach.

## How do we own `fake::Driver` instances?

There are two things that need to access the `fake::Driver` instances: the unit
test case (for configuration and inspection), and the `fake::Kernel` (for
syscall implementations). Therefore, the `fake::Driver` instances must be
accessed through shared references. This requires them to use interior
mutability or be accessed through `RefCell`. Like `fake::Kernel`, we prioritize
the understandability of test code over the implementation of `fake::Driver`, so
`fake::Driver` uses interior mutability.

We could have the `fake::Kernel` own the `fake::Drivers`, but doing so requires
a significant amount of `unsafe` code to satisfy the borrow checker (it really
doesn't like returning references into a struct with interior mutability).
Instead, we use `Rc` again to own the `fake::Drivers`. This results in the
following ownership graph:

```
+-----------+  +--------------+
| Unit test |  | Thread local |
+-----------+  +--------------+
 |       | Rc       : Weak
 |       V          V
 |     +--------------+
 |     | fake::Kernel |
 |     +--------------+
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
