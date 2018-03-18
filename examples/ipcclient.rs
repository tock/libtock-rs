#![feature(alloc)]
#![no_std]
extern crate alloc;
extern crate tock;

use alloc::boxed::Box;
use alloc::string::String;
use tock::console::Console;
use tock::fmt;
use tock::ipc_cs::*;
use tock::timer;

#[allow(unreachable_code)]
// Calls the ipc_server and prints result
fn main() {
    let mut buf: Box<[u8]> = reserve_shared_buffer();
    timer::sleep(timer::Duration::from_ms(1000));

    loop {
        let mut server = ServerHandle::discover_service(String::from("ipcserver")).unwrap();
        let mut payload: [u8; 32] = [5; 32];

        server.share(&mut buf, &mut payload);
        let handle = server.subscribe_callback(|_: usize, _: usize| {
            let mut console = Console::new();
            console.write(String::from("Client: \"Payload: "));
            console.write(fmt::u32_as_hex(buf[0] as u32));
            console.write(String::from("\"\n"));
        });
        server.notify();
        timer::sleep(timer::Duration::from_ms(1000));
        handle.unwrap();
    }
}
