// Libtock regression tests to be used with real hardware.
// Requires P0.03 and P0.04 to be connected (on a nRF52 DK) and
// P0.01 and P0.03 to be connected (on a nRF52840dk).

#![no_std]
extern crate alloc;

use alloc::string::String;
use core::fmt::Write;
use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use libtock::console::ConsoleDriver;
use libtock::gpio::GpioDriverFactory;
use libtock::gpio::GpioState;
use libtock::gpio::ResistorMode;
use libtock::println;
use libtock::result::TockResult;
use libtock::timer::DriverContext;
use libtock::timer::Duration;

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;

    let mut test = LibtockTest::initialize(drivers.console);

    let test_result = libtock_test(&mut test, &mut drivers.timer, &mut drivers.gpio).await;

    if test_result.is_ok() && test.is_success() {
        test.log_success("Test suite finished with state SUCCESS")
    } else {
        test.log_failure("Test suite finished with state FAILURE")
    }
}

#[cfg_attr(
    feature = "__internal_disable_gpio_in_integration_test",
    allow(unused_variables)
)]
async fn libtock_test(
    test: &mut LibtockTest,
    timer: &mut DriverContext,
    gpio: &mut GpioDriverFactory,
) -> TockResult<()> {
    test.console()?;
    test.static_mut()?;
    test.dynamic_dispatch()?;
    test.formatting()?;
    test.heap()?;
    test.drivers_only_instantiable_once()?;
    #[cfg(not(feature = "__internal_disable_timer_in_integration_test"))]
    test.callbacks(timer).await?;
    #[cfg(not(feature = "__internal_disable_gpio_in_integration_test"))]
    test.gpio(gpio)?;
    Ok(())
}

struct LibtockTest {
    success: bool,
}

impl LibtockTest {
    fn initialize(console: ConsoleDriver) -> Self {
        console.create_console();
        Self { success: true }
    }

    fn console(&mut self) -> TockResult<()> {
        self.log_success("Console")
    }

    fn static_mut(&mut self) -> TockResult<()> {
        self.check_if_true(increment_static_mut() == 1, "static mut")
    }

    fn dynamic_dispatch(&mut self) -> TockResult<()> {
        let (x, y) = if foo() == "foo" {
            (&'0' as &dyn MyTrait, &0usize as &dyn MyTrait)
        } else {
            (&0usize as &dyn MyTrait, &'0' as &dyn MyTrait)
        };

        self.check_if_true(
            (x.dispatch(), y.dispatch()) == ("str", "usize"),
            "Dynamic dispatch",
        )
    }

    fn formatting(&mut self) -> TockResult<()> {
        let mut string = String::new();
        write!(string, "{}bar", foo())?;

        self.check_if_true(string == "foobar", "Formatting")
    }

    fn heap(&mut self) -> TockResult<()> {
        let mut string = String::new();
        string.push_str(foo());
        string.push_str("bar");

        self.check_if_true(string == "foobar", "Heap")
    }

    fn drivers_only_instantiable_once(&mut self) -> TockResult<()> {
        self.check_if_true(
            libtock::retrieve_drivers().is_err(),
            "Drivers only instantiable once",
        )
    }

    #[cfg_attr(
        feature = "__internal_disable_timer_in_integration_test",
        allow(dead_code)
    )]
    async fn callbacks(&mut self, timer_context: &mut DriverContext) -> TockResult<()> {
        let mut callback_hit = false;
        let mut with_callback = timer_context.with_callback(|_, _| callback_hit = true);
        let mut timer = with_callback.init()?;

        timer.set_alarm(Duration::from_ms(1000))?;

        AlternatingFuture { yielded: false }.await;

        mem::drop(timer);

        self.check_if_true(callback_hit, "Callbacks")
    }

    #[cfg_attr(
        feature = "__internal_disable_gpio_in_integration_test",
        allow(dead_code)
    )]
    fn gpio(&mut self, gpio: &mut GpioDriverFactory) -> TockResult<()> {
        let mut gpio_driver = gpio.init_driver().ok().unwrap();

        self.log_success("GPIO initialization")?;

        let mut gpios = gpio_driver.gpios();
        let mut pin_in = gpios.next().unwrap();
        let pin_in = pin_in.enable_input(ResistorMode::PullDown).ok().unwrap();
        let mut pin_out = gpios.next().unwrap();
        let pin_out = pin_out.enable_output().ok().unwrap();

        self.log_success("GPIO activation")?;

        pin_out.set_high().ok().unwrap();

        self.check_if_true(
            pin_in.read().ok() == Some(GpioState::High),
            "GPIO read/write",
        )
    }

    fn is_success(&self) -> bool {
        self.success
    }

    fn check_if_true(&mut self, condition: bool, message: &str) -> TockResult<()> {
        if condition {
            self.log_success(message)
        } else {
            self.log_failure(message)
        }
    }

    fn log_success(&mut self, message: &str) -> TockResult<()> {
        println!("[      OK ] {}", message);
        Ok(())
    }

    fn log_failure(&mut self, message: &str) -> TockResult<()> {
        println!("[ FAILURE ] {}", message);
        self.success = false;
        Ok(())
    }
}

#[inline(never)]
// Do not inline this to prevent compiler optimizations
fn foo() -> &'static str {
    "foo"
}

#[inline(never)]
fn increment_static_mut() -> usize {
    static mut STATIC: usize = 0;

    unsafe {
        STATIC += 1;
        STATIC
    }
}

trait MyTrait {
    fn dispatch(&self) -> &'static str;
}

impl MyTrait for usize {
    fn dispatch(&self) -> &'static str {
        "usize"
    }
}

impl MyTrait for char {
    fn dispatch(&self) -> &'static str {
        "str"
    }
}

struct AlternatingFuture {
    yielded: bool,
}

impl Future for AlternatingFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        self.yielded = !self.yielded;
        if self.yielded {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
