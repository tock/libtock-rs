#![no_std]

use libtock::result::TockResult;

fn main() {
    let _ = libtock::LibTock {};
    panic!("Bye world!");
}
