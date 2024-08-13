//! A simple IPC example for using an RNG service.
//!
//! This application uses an RNG IPC service which, on request, will yield a
//! random string of bytes. It prints this random string to the console.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::ipc::{Ipc, IpcCallData, IpcListener};
use libtock::platform::Syscalls;
use libtock::runtime::TockSyscalls;
use libtock::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

const NUM_BYTES: usize = 8;
static mut BUFFER: RngBuffer<NUM_BYTES> = RngBuffer([0; NUM_BYTES]);
const CLIENT_LISTENER: IpcListener<fn(IpcCallData)> = IpcListener(callback);

#[repr(align(8))]
struct RngBuffer<const N: usize>([u8; N]);

struct Randomness<'a>(&'a [u8]);

impl<'a> core::fmt::Display for Randomness<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut bytes = self.0.iter();
        while let Some(&byte) = bytes.next() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

fn callback(data: IpcCallData) {
    writeln!(Console::writer(), "callback thingy: {:?}", data.buffer,).unwrap();
    // writeln!(
    //     Console::writer(),
    //     "CLIENT: Successfully received random bytes {}.",
    //     Randomness(data.buffer.unwrap())
    // )
    // .unwrap();
}

fn main() {
    let rng_service: u32 = match Ipc::discover(b"ipc_service") {
        Ok(service_id) => service_id,
        Err(e) => {
            writeln!(
                Console::writer(),
                "CLIENT: Unable to discover IPC service: {e:?}. Is IPC service installed?"
            )
            .unwrap();
            return;
        }
    };
    Ipc::register_client_listener(rng_service, &CLIENT_LISTENER).unwrap();
    Ipc::share(rng_service, unsafe { &mut BUFFER.0 }).unwrap();

    loop {
        writeln!(
            Console::writer(),
            "CLIENT: Requesting random bytes from {}...",
            rng_service
        )
        .unwrap();
        Ipc::notify_service(rng_service).unwrap();
        TockSyscalls::yield_wait();
        Alarm::sleep_for(Milliseconds(2000)).unwrap();
    }
}
