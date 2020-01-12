// Triggers the panic handler. Should make all LEDs flash.

#![no_std]

use libtock::result::TockResult;

#[libtock::main]
async fn main() -> TockResult<()> {
    panic!("Bye world!");
}
