#![feature(alloc)]
#![no_std]
extern crate alloc;
extern crate tock;

use alloc::string::String;
use alloc::vec::Vec;
use tock::ipc_cs;
use tock::ipc_cs::IpcServerDriver;

#[allow(unreachable_code)]
// Prints the payload and adds one to the first byte.
fn main() {
    let cb = &mut |pid: usize, _: usize, message: &mut [u8]| {
        let filtered = message
            .to_vec()
            .iter()
            .filter(|&x| *x != 0)
            .map(|x| *x)
            .collect::<Vec<u8>>();
        let s = String::from_utf8_lossy(&filtered);
        if s == String::from("client") {
            let m = String::from("test_ipc = \"passed\"\n");
            let b = m.as_bytes();
            let l = b.len();
            message[..l].clone_from_slice(b);
            ipc_cs::notify_client(pid);
        }
    };
    #[allow(unused_variables)]
    let server = IpcServerDriver::start(cb);

    loop {
        tock::syscalls::yieldk();
    }
    server.unwrap();
}
