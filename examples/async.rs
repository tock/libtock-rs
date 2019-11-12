#![no_std]

use core::executor;
use futures::future;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::futures as libtock_futures;
use libtock::led;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    executor::block_on(future::join(blink_periodically(), blink_on_button_press()));
}

async fn blink_periodically() {
    let led = led::get(0).unwrap();
    loop {
        timer::sleep(Duration::from_ms(250)).await;
        led.on();
        timer::sleep(Duration::from_ms(250)).await;
        led.off();
    }
}

async fn blink_on_button_press() {
    let mut with_callback = buttons::with_callback(|_, _| {});
    let mut buttons = with_callback.init().unwrap();
    let mut button = buttons.iter_mut().next().unwrap();
    let button = button.enable().unwrap();
    let led = led::get(1).unwrap();

    loop {
        libtock_futures::wait_until(|| button.read() == ButtonState::Released).await;
        libtock_futures::wait_until(|| button.read() == ButtonState::Pressed).await;
        led.on();
        libtock_futures::wait_until(|| button.read() == ButtonState::Released).await;
        libtock_futures::wait_until(|| button.read() == ButtonState::Pressed).await;
        led.off();
    }
}
