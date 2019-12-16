#![no_std]

use libtock::result::TockResult;
use libtock_support_macros::libtock_main;

#[libtock_main]
async fn main() -> TockResult<()> {
    let _ = libtock::LibTock {};
    panic!("Bye world!");
}
