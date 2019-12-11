#![no_std]

use futures::future;
use libtock::buttons;
use libtock::buttons::ButtonState;
use libtock::futures as libtock_futures;
use libtock::led;
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;

async fn main() -> TockResult<()> {
    future::try_join(blink_periodically(), blink_on_button_press()).await?;
    Ok(())
}

async fn blink_periodically() -> TockResult<()> {
    let led = led::get(0).unwrap();
    loop {
        timer::sleep(Duration::from_ms(250)).await?;
        led.on()?;
        timer::sleep(Duration::from_ms(250)).await?;
        led.off()?;
    }
}

async fn blink_on_button_press() -> TockResult<()> {
    let mut with_callback = buttons::with_callback(|_, _| {});
    let mut buttons = with_callback.init()?;
    let mut button = buttons.iter_mut().next().unwrap();
    let button = button.enable()?;
    let led = led::get(1).unwrap();

    loop {
        libtock_futures::wait_until(|| button.read().ok() == Some(ButtonState::Released)).await;
        libtock_futures::wait_until(|| button.read().ok() == Some(ButtonState::Pressed)).await;
        led.on()?;
        libtock_futures::wait_until(|| button.read().ok() == Some(ButtonState::Released)).await;
        libtock_futures::wait_until(|| button.read().ok() == Some(ButtonState::Pressed)).await;
        led.off()?;
    }
}
