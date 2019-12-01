#![no_std]

/// Hardware regression tests.
/// Need P0.03 and P0.04 to be connected (on a nrf52-dk).
extern crate alloc;

use alloc::string::String;
use core::fmt::Write;
use futures::future;
use libtock::console::Console;
use libtock::gpio::GpioRead;
use libtock::gpio::GpioState;
use libtock::gpio::GpioWrite;
use libtock::gpio::ResistorMode;
use libtock::result::TockResult;
use libtock::timer::DriverContext;
use libtock::timer::Duration;
use libtock::Drivers;

static mut STATIC: usize = 0;

trait MyTrait {
    fn do_something_with_a_console(&self, console: &mut Console);
}

impl MyTrait for usize {
    fn do_something_with_a_console(&self, console: &mut Console) {
        writeln!(console, "trait_obj_value_usize = {}", &self).unwrap();
    }
}

impl MyTrait for String {
    fn do_something_with_a_console(&self, console: &mut Console) {
        writeln!(console, "trait_obj_value_string = {}", &self).unwrap();
    }
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let Drivers {
        mut gpio_driver_factory,
        mut timer_context,
        console_driver,
        ..
    } = libtock::retrieve_drivers()?;

    let mut gpio_driver = gpio_driver_factory.init_driver()?;
    let mut console = console_driver.create_console();

    let mut gpios = gpio_driver.gpios();
    let mut pin_in = gpios.next().unwrap();
    let pin_in = pin_in.enable_input(ResistorMode::PullDown)?;
    let mut pin_out = gpios.next().unwrap();
    let mut pin_out = pin_out.enable_output()?;

    writeln!(console, "[test-results]")?;

    test_heap(&mut console);
    test_formatting(&mut console);
    test_static_mut(&mut console);

    test_gpio(&mut console, &pin_in, &mut pin_out);

    test_trait_objects(&mut console, &pin_in, &mut pin_out)?;

    test_callbacks_and_wait_forever(&mut console, &mut timer_context).await
}

fn test_heap(console: &mut Console) {
    let mut string = String::from("heap_test = \"Heap ");
    string.push_str("works.\"");
    writeln!(console, "{}", string).unwrap();
}

fn test_formatting(console: &mut Console) {
    writeln!(console, "formatting =  {}", String::from("works")).unwrap();
}

/// needs P0.03 and P0.04 to be connected
/// Output order should be:
/// trait_obj_value_usize = 1
/// trait_obj_value_string = string
fn test_trait_objects(
    console: &mut Console,
    pin_in: &GpioRead,
    pin_out: &mut GpioWrite,
) -> TockResult<()> {
    pin_out.set_high()?;

    let string = String::from("string");

    let x = match pin_in.read()? {
        GpioState::Low => &string as &dyn MyTrait,
        GpioState::High => &1usize as &dyn MyTrait,
    };

    let y = match pin_in.read()? {
        GpioState::Low => &1usize as &dyn MyTrait,
        GpioState::High => &string as &dyn MyTrait,
    };

    x.do_something_with_a_console(console);
    y.do_something_with_a_console(console);

    Ok(())
}

fn test_static_mut(console: &mut Console) {
    increment_static_mut();

    writeln!(console, "should_be_one = {}", unsafe { STATIC }).unwrap();
}

/// needs P0.03 and P0.04 to be connected
fn test_gpio(console: &mut Console, pin_in: &GpioRead, pin_out: &mut GpioWrite) {
    pin_out.set_high().ok().unwrap();

    writeln!(
        console,
        "gpio_works = {}",
        pin_in.read().ok() == Some(GpioState::High)
    )
    .unwrap();
}

async fn test_callbacks_and_wait_forever(
    console: &mut Console,
    timer_context: &mut DriverContext,
) -> TockResult<()> {
    let mut with_callback = timer_context.with_callback(|_, _| {
        writeln!(console, "callbacks_work = true").unwrap();
        writeln!(console, "all_tests_run = true").unwrap();
    });

    let mut timer = with_callback.init()?;

    timer.set_alarm(Duration::from_ms(500))?;

    future::pending().await
}

#[inline(never)]
fn increment_static_mut() {
    unsafe { STATIC += 1 };
}
