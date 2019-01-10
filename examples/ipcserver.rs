#![feature(alloc)]
#![no_std]

extern crate alloc;

use alloc::string::String;
use tock::console::Console;
use tock::fmt::*;
use tock::ipc;
use tock::ipc::IpcServerCallback;
use tock::ipc::IpcServerDriver;

#[allow(unreachable_code)]
// Prints the payload and adds one to the first byte.
fn main() {
    let mut console = Console::new();
    console.write(String::from("Start service:\n"));

    let mut callback = IpcServerCallback::new(|pid: usize, _: usize, message: &mut [u8]| {
        console.write(String::from("Server: \"Payload: "));

        console.write(u32_as_hex(message[0] as u32));
        console.write(String::from("\"\n"));
        message[0] += 1;
        ipc::notify_client(pid);
    });

    let _server = IpcServerDriver::start(&mut callback);

    loop {
        tock::syscalls::yieldk();
    }

    _server.unwrap();
}
