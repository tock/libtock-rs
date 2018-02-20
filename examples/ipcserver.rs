#![feature(alloc)]
#![no_std]
extern crate alloc;
extern crate tock;

use alloc::string::String;
use tock::console::Console;
use tock::fmt::*;
use tock::ipc_cs;
use tock::ipc_cs::IpcServerDriver;

#[allow(unreachable_code)]
// Prints the payload and adds one to the first byte.
fn main() {
    let mut console = Console::new();
    console.write(String::from("Start service:\n"));

    let cb = &mut |pid: usize, _: usize, message: &mut [u8; 32]| {
        console.write(String::from("Server: \"Payload: "));

        console.write(u32_as_hex(message[0] as u32));
        console.write(String::from("\"\n"));
        message[0] += 1;
        ipc_cs::notify_client(pid);
    };
    #[allow(unused_variables)]
    let server = IpcServerDriver::start(cb);

    loop {
        tock::syscalls::yieldk();
    }
    server.unwrap();
}
