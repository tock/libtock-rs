#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use tock::console::Console;
use tock::ipc_cs;
use tock::ipc_cs::ServerHandle;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut buf: Box<[u8]> = ipc_cs::reserve_shared_buffer();
    let mut console = Console::new();
    timer::sleep(Duration::from_ms(3000));

    console.write(String::from("[test-results]\n"));
    let mut string = String::from("heap_test = \"Heap ");
    string.push_str("works.\"\n");
    console.write(string);

    let mut server = ServerHandle::discover_service(String::from("hardware_test_server")).unwrap();
    let mut payload: [u8; 32] = [0; 32];
    let m = String::from("client");
    let b = m.as_bytes();
    let l = b.len();
    payload[..l].clone_from_slice(b);

    server.share(&mut buf, &mut payload);

    server.subscribe_callback(|_: usize, _: usize| {
        let filtered = buf.to_vec()
            .iter()
            .filter(|&x| *x != 0)
            .map(|x| *x)
            .collect::<Vec<u8>>();
        let s = String::from_utf8_lossy(&filtered);
        console.write(String::from(s).clone());
        console.write(String::from("test=\"done\"\n"));
    });
    server.notify();

    for _ in 0.. {
        timer::sleep(Duration::from_ms(500))
    }
}
