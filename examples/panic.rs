#![no_std]

use libtock::result::TockResult;

#[libtock::main]
async fn main() -> TockResult<()> {
    let _ = libtock::LibTock {};
    panic!("Bye world!");
}
