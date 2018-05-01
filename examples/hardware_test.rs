#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::string::String;
use alloc::vec::Vec;
use tock::console::Console;
use tock::ipc;
use tock::ipc::IpcClientCallback;
use tock::ipc::ServerHandle;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut server_buf = ipc::reserve_shared_buffer();
    let mut my_buf = ipc::reserve_shared_buffer();

    let mut console = Console::new();

    // This sleep is neccessary to assure, that during installation of
    // the client/server pair the tests are only run once.
    timer::sleep(Duration::from_ms(3000));

    console.write(String::from("[test-results]\n")).unwrap();
    let mut string = String::from("heap_test = \"Heap ");
    string.push_str("works.\"\n");
    console.write(string).unwrap();

    let mut server = ServerHandle::discover_service(String::from("hardware_test_server")).unwrap();
    let mut payload: [u8; 32] = [0; 32];
    let m = String::from("client");
    let b = m.as_bytes();
    let l = b.len();
    payload[..l].clone_from_slice(b);

    let mut handle = server.share(&mut server_buf).unwrap();
    handle.write_bytes(&payload);

    let mut callback = IpcClientCallback::new(|_: usize, _: usize| {
        handle.read_bytes(&mut my_buf.buffer);
        let filtered = my_buf
            .buffer
            .iter()
            .cloned()
            .filter(|&x| x != 0)
            .collect::<Vec<_>>();
        let s = String::from_utf8_lossy(&filtered);
        console.write(String::from(s).clone()).unwrap();
        console.write(String::from("test=\"done\"\n")).unwrap();
    });

    let handle = server.subscribe_callback(&mut callback);
    server.notify().unwrap();

    for _ in 0.. {
        timer::sleep(Duration::from_ms(500))
    }
    handle.unwrap();
}
