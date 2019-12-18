#![no_std]
/**
 * This program is a proof-of-concept to easily schedule many callbacks for an application, and
 * show how that can trigger a memory fault if the Tock kernel doesn't unschedule pending callbacks
 * when unsubscribing.
 *
 * To test whether your kernel unschedules pending callbacks properly, install this app and press
 * all the buttons quickly. This should trigger a panic in this app if your kernel doesn't include
 * https://github.com/tock/tock/pull/1376. You can also manually re-introduce the bug and check with
 * this app by removing the `remove_pending_callbacks` line in `kernel/src/sched.rs` of your Tock
 * kernel.
 **/
use core::cell::Cell;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::rng;
use libtock::syscalls;

fn main() {
    loop {
        wait_button();
        rng();
    }
}

// Don't inline this function, so that the stack frame doesn't align with rng().
#[inline(never)]
fn wait_button() {
    // Listen to the button presses.
    let touched = Cell::new(false);
    let mut buttons_callback = buttons::with_callback(|_button_num, state| {
        match state {
            // Update the touch status. When called from rng()'s stack frame, this write will
            // corrupt the stack.
            ButtonState::Pressed => touched.set(true),
            ButtonState::Released => (),
        };
    });
    let mut buttons = buttons_callback.init().unwrap();
    for mut button in &mut buttons {
        button.enable().unwrap();
    }

    // Wait for a button touch.
    syscalls::yieldk_for(|| {
        // Busy loop to make this slow and allow the user to schedule many button press callbacks.
        let mut x = 0;
        for i in 0..10000000 {
            unsafe { core::ptr::write_volatile(&mut x, i) };
        }
        touched.get()
    });

    // Cleanup callbacks.
    for mut button in &mut buttons {
        button.disable().unwrap();
    }
}

// Inline this function within main()'s stack frame.
#[inline(always)]
fn rng() {
    // Just some other driver that uses a different callback and calls yieldk_for().
    rng::fill_buffer(&mut [0; 32]);
}
