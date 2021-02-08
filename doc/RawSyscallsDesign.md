`RawSyscalls` Design
====================

## High-Level Overview

`libtock_platform` has two traits for Tock's system calls. `Syscalls` is the
high-level interface that drivers should use to make system calls. `RawSyscalls`
is a low-level interface to the system calls that is used to switch between real
kernel system calls (provided by `libtock_runtime::TockSyscalls`) and unit
tests' fake system call implementation (provided by
`libtock_unittest::fake::Kernel`).

Code that relies on `RawSyscalls` can be unit tested by using `fake::Kernel`,
but the code in the `RawSyscalls` implementation cannot be unit tested. To
minimize the amount of non-unit-testable code, `RawSyscalls` is designed to be a
minimal wrapper around `asm!`.

## Major Design Considerations

**Note: This file uses the same register naming convention as the Tock 2.0
syscalls TRD. Registers `r0`-`r4` refer to ARM registers `r0`-`r4` and RISC-V
registers `a0`-`a4`.**

1. Most system calls only clobber `r0`-`r4`, while Yield calls have a far longer
   clobber list. It would be inefficient to share an `asm!` call between Yield
   and another system call.
1. Exit does not return. This initially seems to be an important distinction,
   but it is not. `Syscalls` can use `unreachable_unchecked!` to tell the
   compiler it doesn't exit, and it should be obvious the return values are all
   unused.
1. The compiler is unable to optimize away unused arguments. For example,
   Memop's "get process RAM start address" operation only needs `r0` set, while
   Memop's "break" operation needs both `r0` and `r1` set. If our inline
   assembly calls "get process RAM start address" but sets both `r0` and `r1`,
   the compiler doesn't know that `r1` will be ignored so setting that register
   will not be optimized away. Therefore we want to set the minimum number of
   argument registers possible.
1. The cost of specifying unused return registers is only that of unnecessarily
   marking a register as clobbered. Explanation: After inlining, an unused
   register is marked as "changed by the assembly" but can immediately be
   re-used by the compiler, which is the same as a clobbered register. System
   calls should generally be inlined -- and even if they aren't, the unused
   return values will probably be passed in caller-saved registers (this is true
   for the C ABI, so probably true for the Rust ABI), which are treated as
   clobbered regardless.
1. Command always needs to set all four registers, because otherwise we risk
   leaking data to capsules (which are untrusted for confidentiality). Exit,
   Memop, and Yield do not have this concern because they are implemented by the
   core kernel (which is trusted).

## Yield Method Breakdown

`yield-wait` does not take any arguments, while `yield-no-wait` takes 1.
Therefore, to avoid setting arguments unnecessarily, `yield-wait` and
`yield-no-wait` should be distinct functions.

## Remaining Method Breakdown

After taking care of `yield`, we break down the remaining system calls by the
number of arguments they take. Here is the breakdown (Exit and Memop are listed
twice because they have sub-operations with varying number of arguments).

* **Zero:**
* **One:** Exit, Memop
* **Two:** Exit, Memop
* **Three:**
* **Four:** Command, Read-Only Allow, Read-Write Allow, Subscribe

Therefore we need three more methods: `one_arg_syscall`, `two_arg_syscall`,
`four_arg_syscall`.

## `u32` versus `usize`

The decision of where to use `u32` and where to use `usize` can be a bit tricky.
The Tock syscall ABI is currently only specified for 32-bit systems, so on real
Tock systems both types match the size of a register, but the unit test
environment can be either 32 bit or 64 bit. `RawSyscalls` uses `usize` for
values that can contain pointers, so that pointers are not truncated in the unit
test environment. To keep types as consistent as possible, it uses `u32` for
register-sized values that cannot be pointers.

## Yield Methods Design

Rust does not specify the representation of `bool`, only that it casts to `0`
and `1`, so it would not be correct use a `bool` as the flag for
`yield-no-wait`. Instead we use a `u8`.

To avoid name collisions with the Yield methods in `Syscalls`, we prepend `raw_`
to their names.

This results in the following signatures:

```rust
raw_yield_no_wait(flag: &mut u8);
raw_yield_wait();
```

Note that the signature of `raw_yield_no_wait` gives up the following two
optimization to reduce its use of `unsafe`:

1. It requires `flag` to be initialized before calling into the kernel, which
   the kernel does not require.
1. Converting the value of `flag` to a bool requires a comparison. Using a
   custom enum here would remove the comparison (assuming `bool` is represented
   via `false = 0` and `true = 1`).

The `asm!` statements for these methods must:

1. Call syscall class `0`.
1. `yield_no_wait`: pass `0` in `r0` as `inlateout`; `yield_wait`: pass `1` in
   `r0` as `inlateout`.
1. `yield_no_wait`: pass the `flag` reference in `r1` as `inlateout`.
1. Mark all other caller-saved registers as `lateout` clobbers.
1. NOT provide any of the following options:
   * `pure`: Yield has side effects
   * `nomem`: A callback can read + write globals
   * `readonly`: A callback can write globals
   * `preserves_flags`: A callback can change flags
   * `noreturn`: Yield is expected to return
   * `nostack`: A callback needs the stack

## `one_arg_syscall` and `two_arg_syscall` Design

None of the Exit and Memop operations that currently exist can produce memory
unsafety, and in principle it is possible to write `one_arg_syscall` and
`two_arg_syscall` in a way that doesn't require them to be `unsafe`. However,
encoding the safe combinations of syscall class and operation number can only be
soundly done using an `unsafe` trait, and doesn't save any `unsafe` uses
relative to making `one_arg_syscall` and `two_arg_syscall` `unsafe`. Therefore
we take the simple option and make them `unsafe`.

Memop returns pointers in r1, so we need to return a `usize` for `r1` instead of
a `u32`.

This results in the following signatures:

```rust
unsafe fn one_arg_syscall(op: u32, class: u8) -> (ReturnVariant, usize);
unsafe fn two_arg_syscall(op: u32, r1: usize, class: u8) -> (ReturnVariant, usize);
```

The `asm!` statements for these methods must:

1. Call the provided system call class.
1. Specify `r0` as an `inlateout` register.
1. `one_arg_syscall`: specify `r1` as a `lateout` register; `two_arg_syscall`:
   specify `r1` as a `inlateout` register.
1. Return `(r0, r1)`
1. Do not mark any registers as clobbered.
1. Have the following options:
   * `preserves_flags`
   * `nostack`
   * `nomem`: It is okay for the compiler to cache globals across Memop calls
1. Does NOT have any of the following options:
   * `pure`
   * `readonly`: Incompatible with `nomem`
   * `noreturn`: True for Exit but not Memop

## `four_arg_syscall` Design

Read-Only Allow, Read-Write Allow, and Subscribe can all cause memory unsafety,
so we don't need to try to make a safe interface for them.

**Inputs:** Driver IDs and buffer/command/subscription IDs are all 32-bit and are passed in
`r0` and `r1`. Read-Only and Read-Write Allow pass pointer-sized data in `r2`,
so it must be a `usize`. Subscribe's application data pointer can contain a
pointer, so the `r3` input must be a `usize` as well.

**Outputs:** `r0` always contains an arbitrary `ReturnVariant`. Subscribe can
return pointers in `r1`, `r2`, and `r3`, so they must always be a `usize`.

For Subscribe, the callback pointer should either be `0` (for the Null
Callback) or an `unsafe extern fn(u32, u32, u32, usize)`.

This gives us the following signature for `four_arg_syscall`:

```rust
unsafe fn four_arg_syscall(
    r0: u32,
    r1: u32,
    r2: usize,
    r3: usize,
    class: u8)
-> (ReturnVariant, usize, usize, usize);
```

The `asm!` statement in `four_arg_syscall` must:

1. Call the syscall class specified by `class`
1. Pass `r0`-`r3` in the corresponding registers as `inlateout` registers.
1. Return `r0`-`r3` in order.
1. Not mark any registers as clobbered.
1. Have all of the following options:
   * `preserves_flags`
   * `nostack`
1. Not have any of the following options:
   * `pure`
   * `nomem`: The compiler must write to globals before Allow
   * `readonly`: Read-Write Allow can modify memory
   * `noreturn`
