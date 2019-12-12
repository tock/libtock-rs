#![no_std]

use libtock::result::TockResult;

fn main() -> TockResult<()> {
    let _ = libtock::LibTock {};
    panic!("Bye world!");
}
