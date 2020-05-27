#![no_std]

use core::fmt::Write;
use libtock::hmac::{HmacDataBuffer, HmacDestBuffer, HmacKeyBuffer};
use libtock::result::TockResult;
use libtock::syscalls;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;
    let mut console = drivers.console.create_console();
    writeln!(console, "Starting HMAC example")?;
    let hmac_driver = drivers.hmac.init_driver()?;

    writeln!(console, "Loading in 0 key")?;
    let mut key_buffer = HmacKeyBuffer::default();
    let _key_buffer = hmac_driver.init_key_buffer(&mut key_buffer)?;
    writeln!(console, "  done")?;

    writeln!(console, "Creating data buffer")?;
    let mut data_buffer = HmacDataBuffer::default();
    let data: &[u8; 72] =
        b"A language empowering everyone to build reliable and efficient software.";

    for (i, d) in data.iter().enumerate() {
        data_buffer.buffer[i] = *d;
    }
    let _data_buffer = hmac_driver.init_data_buffer(&mut data_buffer)?;
    writeln!(console, "  done")?;

    writeln!(console, "Creating dest buffer")?;
    let mut dest_buffer = HmacDestBuffer::default();
    let dest_buffer = hmac_driver.init_dest_buffer(&mut dest_buffer)?;
    writeln!(console, "  done")?;

    let mut temp_buffer = [0; libtock::hmac::DEST_BUFFER_SIZE];

    writeln!(console, "Setting callback and running")?;
    let mut callback = |_result, _digest| {
        writeln!(console, "HMAC Complete, printing digest").unwrap();
        dest_buffer.read_bytes(&mut temp_buffer[..]);

        for buf in temp_buffer.iter().take(libtock::hmac::DEST_BUFFER_SIZE) {
            write!(console, "{:x}", *buf).unwrap();
        }
    };

    let _subscription = hmac_driver.subscribe(&mut callback)?;

    hmac_driver.run()?;

    loop {
        unsafe { syscalls::raw::yieldk() };
    }
}
