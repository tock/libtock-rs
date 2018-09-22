#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use core::fmt::Write;
use tock::console::Console;
use tock::ipc;
use tock::ipc::IpcServerCallback;
use tock::ipc::IpcServerDriver;

#[allow(unreachable_code)]
// Prints the payload and adds one to the first byte.
fn main() {
    let mut console = Console::new();
    writeln!(console, "Start service:");

    let mut callback = IpcServerCallback::new(|pid: usize, _: usize, message: &mut [u8]| {
        writeln!(console, "Server: \"Payload: {}\"", message[0]);
        message[0] += 1;
        ipc::notify_client(pid);
    });

    let _server = IpcServerDriver::start(&mut callback);

    loop {
        tock::syscalls::yieldk();
    }

    _server.unwrap();
}
