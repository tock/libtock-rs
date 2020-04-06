use core::cell::Cell;
use core::cell::RefCell;
use std::vec::Vec;

/// yield for a callback fired by the kernel
///
/// # Safety
///
/// Yielding in the main function should be safe. Nevertheless, yielding manually
/// is not required as this is already achieved by the `async` runtime.
///
/// When yielding in callbacks, two problems can arise:
/// - The guarantees of `FnMut` are violated. In this case, make sure your callback has `Fn` behavior.
/// - Callbacks can get executed in a nested manner and overflow the stack quickly.
pub unsafe fn yieldk() {
    EVENTS.with(|e| e.borrow_mut().push(Event::YieldK));
}

/// Subscribe a callback to the kernel
/// # Safety
/// Unsafe as passed callback is dereferenced and called.
pub unsafe fn subscribe(
    arg1: usize,
    arg2: usize,
    arg3: *const unsafe extern "C" fn(usize, usize, usize, usize),
    arg4: usize,
) -> isize {
    EVENTS.with(|e| {
        e.borrow_mut()
            .push(Event::Subscribe(arg1, arg2, arg3, arg4))
    });
    NEXT_OUTPUT.with(|e| e.get())
}

/// Send a command to the tock kernel
/// # Safety
/// This function usually involves assembly calls which are unsafe.
pub unsafe fn command(arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> isize {
    EVENTS.with(|e| e.borrow_mut().push(Event::Command(arg1, arg2, arg3, arg4)));
    NEXT_OUTPUT.with(|e| e.get())
}

/// Call a command only taking into accoun the first argument
/// # Safety
/// Unsafe as ignored arguments cause leaking of registers to the kernel
pub unsafe fn command1(arg1: usize, arg2: usize, arg3: usize) -> isize {
    EVENTS.with(|e| e.borrow_mut().push(Event::Command1(arg1, arg2, arg3)));
    NEXT_OUTPUT.with(|e| e.get())
}

/// Share a memory region with the kernel
/// # Safety
/// Unsafe as the pointer to the shared buffer is potentially dereferenced by the kernel.
pub unsafe fn allow(arg1: usize, arg2: usize, arg3: *mut u8, arg4: usize) -> isize {
    EVENTS.with(|e| e.borrow_mut().push(Event::Allow(arg1, arg2, arg3, arg4)));
    NEXT_OUTPUT.with(|e| e.get())
}

/// Generic operations on the app's memory as requesting more memory
/// # Safety
/// Allows the kernel to do generic operations on the app's memory which can cause memory corruption.
pub unsafe fn memop(arg1: u32, arg2: usize) -> isize {
    EVENTS.with(|e| e.borrow_mut().push(Event::Memop(arg1, arg2)));
    NEXT_OUTPUT.with(|e| e.get())
}

/// For tests: Run the closure recording the syscalls which are invoked in during the run of the closure.
pub fn run_recording_events<R, C: FnMut(&NextReturn) -> R>(mut f: C) -> Vec<Event> {
    NEXT_OUTPUT.with(|n| n.set(0));
    NEXT_OUTPUT.with(|n| f(n));
    let mut output = Vec::new();
    EVENTS.with(|e| output.append(&mut e.borrow_mut()));
    output
}

thread_local!(static EVENTS: RefCell<Vec<Event>> = RefCell::new(Vec::new()));
thread_local!(static NEXT_OUTPUT: NextReturn = NextReturn { next_return: Cell::new(0) });

#[derive(Clone, Debug, PartialEq)]
/// For tests: syscall event
pub enum Event {
    YieldK,
    Subscribe(
        usize,
        usize,
        *const unsafe extern "C" fn(usize, usize, usize, usize),
        usize,
    ),
    Command(usize, usize, usize, usize),
    Command1(usize, usize, usize),
    Allow(usize, usize, *mut u8, usize),
    Memop(u32, usize),
}

/// For tests: controls the next return value of any syscall
pub struct NextReturn {
    next_return: Cell<isize>,
}

impl NextReturn {
    /// Set the next return value
    pub fn set(&self, value: isize) {
        self.next_return.set(value);
    }

    fn get(&self) -> isize {
        self.next_return.get()
    }
}
