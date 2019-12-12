#![no_std]

use libtock::result::TockResult;

libtock::async_main!(async_main);
fn async_main() -> TockResult<()> {
    let _ = libtock::LibTock {};
    panic!("Bye world!");
}
