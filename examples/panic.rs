#![no_std]

use libtock::result::TockResult;

async fn main() -> TockResult<()> {
    let _ = libtock::LibTock {};
    panic!("Bye world!");
}
