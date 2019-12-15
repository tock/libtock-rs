#![no_std]

use libtock_support_macros::libtock_main;

#[libtock_main]
async fn main() -> libtock::result::TockResult<()> {
    let _ = libtock::LibTock {};
    panic!("Bye world!");
}
