//! A simple IPC example for setting up an RNG IPC service.
//!
//! This application sets up an RNG IPC service which, on request, will yield
//! a random string of bytes via a provided shared memory region.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::console::Console;
use libtock::ipc::{Ipc, IpcCallData, IpcListener};
use libtock::platform::Syscalls;
use libtock::rng::Rng;
use libtock::runtime::TockSyscalls;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

const NUM_BYTES: u32 = 8;
const SERVICE_LISTENER: IpcListener<fn(IpcCallData)> = IpcListener(callback);

fn callback(data: IpcCallData) {
    writeln!(
        Console::writer(),
        "SERVICE: Request received, generating random bytes..."
    )
    .unwrap();

    writeln!(Console::writer(), "SERVICE: {:?}", data.buffer).unwrap();

    Rng::get_bytes_sync(data.buffer.unwrap(), NUM_BYTES).unwrap();
    Ipc::notify_client(data.caller_id).unwrap();
}

fn main() {
    Ipc::register_service_listener(b"ipc_service", &SERVICE_LISTENER).unwrap();
    loop {
        TockSyscalls::yield_wait();
    }
}
