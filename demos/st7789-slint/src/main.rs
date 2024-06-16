//! This sample demonstrates displaying a slint GUI on a ST7789 display
//! using a rust-embedded based crate

#![no_main]
#![no_std]

extern crate alloc;

use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::gpio::Gpio;
use libtock::platform::ErrorCode;
use libtock::runtime::{set_main, stack_size};
use libtock::spi_controller::EmbeddedHalSpi;

use critical_section::RawRestoreState;
use embedded_alloc::Heap;

use display_interface_spi::SPIInterface;
use embedded_hal::digital::OutputPin;
use mipidsi::{models::ST7789, options::ColorInversion, Builder, Display};

slint::include_modules!();

set_main! {main}
stack_size! {0x1400}

#[global_allocator]
static HEAP: Heap = Heap::empty();

struct MyCriticalSection;
critical_section::set_impl!(MyCriticalSection);

unsafe impl critical_section::Impl for MyCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        // Tock is single threaded, so this can only be preempted by interrupts
        // The kernel won't schedule anything from our app unless we yield
        // so as long as we don't yield we won't concurrently run with
        // other critical sections from our app.
        // The kernel might schedule itself or other applications, but there
        // is nothing we can do about that.
    }

    unsafe fn release(_token: RawRestoreState) {}
}

// Setup the heap and the global allocator.
unsafe fn setup_heap() {
    use core::mem::MaybeUninit;

    const HEAP_SIZE: usize = 1024 * 6;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

// Display
const W: i32 = 240;
const H: i32 = 240;

struct Delay;

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        Alarm::sleep_for(Milliseconds(ns / (1000 * 1000))).unwrap();
    }
}

#[mcu_board_support::entry]
fn main() {
    writeln!(Console::writer(), "st7789-slint: example\r").unwrap();

    unsafe {
        setup_heap();
    }

    // Configure platform for Slint
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
    );

    window.set_size(slint::PhysicalSize::new(240, 240));

    slint::platform::set_platform(alloc::boxed::Box::new(SlintPlatform {
        window: window.clone(),
    }))
    .unwrap();

    MainWindow::new().unwrap().run().unwrap();
}

struct SlintPlatform {
    window: alloc::rc::Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
}

impl slint::platform::Platform for SlintPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<alloc::rc::Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(Alarm::get_milliseconds().unwrap())
    }

    fn debug_log(&self, arguments: core::fmt::Arguments) {
        writeln!(Console::writer(), "{}\r", arguments).unwrap();
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        let mut gpio_dc = Gpio::get_pin(0).unwrap();
        let dc = gpio_dc.make_output().unwrap();

        let mut gpio_reset = Gpio::get_pin(1).unwrap();
        let reset = gpio_reset.make_output().unwrap();

        let di = SPIInterface::new(EmbeddedHalSpi, dc);

        let mut delay = Delay;
        let display = Builder::new(ST7789, di)
            .display_size(W as u16, H as u16)
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(reset)
            .init(&mut delay)
            .unwrap();

        let mut buffer_provider = DrawBuffer {
            display,
            buffer: &mut [slint::platform::software_renderer::Rgb565Pixel(100); 240],
        };

        writeln!(Console::writer(), "Setup platform, running loop\r").unwrap();

        loop {
            slint::platform::update_timers_and_animations();

            self.window.draw_if_needed(|renderer| {
                renderer.render_by_line(&mut buffer_provider);
            });

            if self.window.has_active_animations() {
                continue;
            }

            if let Some(duration) = slint::platform::duration_until_next_timer_update() {
                Alarm::sleep_for(Milliseconds(duration.as_millis() as u32)).unwrap();
            }
        }
    }
}

struct DrawBuffer<'a, Display> {
    display: Display,
    buffer: &'a mut [slint::platform::software_renderer::Rgb565Pixel],
}

impl<DI: display_interface::WriteOnlyDataCommand, RST: OutputPin<Error = ErrorCode>>
    slint::platform::software_renderer::LineBufferProvider
    for &mut DrawBuffer<'_, Display<DI, mipidsi::models::ST7789, RST>>
{
    type TargetPixel = slint::platform::software_renderer::Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [slint::platform::software_renderer::Rgb565Pixel]),
    ) {
        let buffer = &mut self.buffer[range.clone()];

        render_fn(buffer);

        // We send empty data just to get the device in the right window
        self.display
            .set_pixels(
                range.start as u16,
                line as _,
                range.end as u16,
                line as u16,
                buffer
                    .iter()
                    .map(|x| embedded_graphics::pixelcolor::raw::RawU16::new(x.0).into()),
            )
            .unwrap();
    }
}
