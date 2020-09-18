#![no_std]
/// This is a very basic CTAP example
/// This example only calls the CTAP driver calls, it does not implement CTAP
use core::fmt::Write;
use libtock::ctap::{CtapRecvBuffer, CtapSendBuffer};
use libtock::result::TockResult;
use libtock::syscalls;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;
    let mut console = drivers.console.create_console();
    writeln!(console, "Starting CTAP example")?;
    let ctap_driver = drivers.ctap.init_driver()?;

    writeln!(console, "Creating recv buffer")?;
    let mut recv_buffer = CtapRecvBuffer::default();
    let recv_buffer = ctap_driver.init_recv_buffer(&mut recv_buffer)?;
    writeln!(console, "  done")?;

    writeln!(console, "Creating send buffer")?;
    let mut send_buffer = CtapSendBuffer::default();
    let _send_buffer = ctap_driver.init_send_buffer(&mut send_buffer)?;
    writeln!(console, "  done")?;

    let mut temp_buffer = [0; libtock::ctap::RECV_BUFFER_SIZE];

    writeln!(console, "Setting callback and running")?;
    let mut callback = |_, _| {
        writeln!(console, "CTAP Complete, printing data").unwrap();
        recv_buffer.read_bytes(&mut temp_buffer[..]);

        for buf in temp_buffer.iter().take(libtock::ctap::RECV_BUFFER_SIZE) {
            write!(console, "{:x}", *buf).unwrap();
        }

        let _ret = ctap_driver.allow_receive();
    };

    let _subscription = ctap_driver.subscribe(&mut callback)?;
    ctap_driver.allow_receive()?;

    loop {
        unsafe { syscalls::raw::yieldk() };
    }
}
