use crate::futures;
use crate::result::TockResult;
use crate::syscalls;
use core::cell::Cell;
use core::mem;

const DRIVER_NUMBER: usize = 0x40001;

mod command_nr {
    pub const REQUEST_RNG: usize = 1;
}

mod subscribe_nr {
    pub const BUFFER_FILLED: usize = 0;
}

mod allow_nr {
    pub const SHARE_BUFFER: usize = 0;
}

pub async fn fill_buffer(buf: &mut [u8]) -> TockResult<()> {
    let buf_len = buf.len();

    let shared_memory = syscalls::allow(DRIVER_NUMBER, allow_nr::SHARE_BUFFER, buf)?;

    let is_filled = Cell::new(false);
    let mut is_filled_alarm = |_, _, _| is_filled.set(true);
    let subscription = syscalls::subscribe(
        DRIVER_NUMBER,
        subscribe_nr::BUFFER_FILLED,
        &mut is_filled_alarm,
    )?;

    syscalls::command(DRIVER_NUMBER, command_nr::REQUEST_RNG, buf_len, 0)?;

    futures::wait_until(|| is_filled.get()).await;

    mem::drop(subscription);
    mem::drop(shared_memory);

    Ok(())
}
