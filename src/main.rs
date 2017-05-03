#![feature(asm,lang_items,const_fn)]
#![no_main]
#![no_std]

pub mod arm;

static mut T : u32 = 1234;

#[inline(never)]
#[no_mangle]
pub fn _start() -> ! {
    loop {
        unsafe {
            let x = &mut T;
            let y = *x;
            if y < 1 {
                asm!("nop");
            }
            *x += y;
        }
    }
}
