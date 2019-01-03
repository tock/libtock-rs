// The stack unwinding ABI in ARM is specified as part of EABI. Although
// libunwind has not been ported to Tock OS (and likely will not be), LLVM still
// assumes some of the symbols are present. This causes linking errors
// (undefined symbol __aeabi_unwind_cpp_pr...). In environments without
// libunwind, it appears to be common practice to declare the symbols as empty
// functions; for example, the Linux Kernel does so in arch/arm/kernel/unwind.c.
// We do so here as well. The addition of these symbols to libtock-rs was
// discussed at https://groups.google.com/forum/#!topic/tock-dev/eov8fJmskLk.
#[cfg(target_arch = "arm")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {}

#[cfg(target_arch = "arm")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr1() {}

#[cfg(target_arch = "arm")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr2() {}
