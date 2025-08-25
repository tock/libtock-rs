#![no_main]
#![no_std]
use core::cell::Cell;
use core::fmt::Write;
use libtock::buttons::{ButtonListener, ButtonState, Buttons};
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size, TockSyscalls};
use libtock_platform::share;
use libtock_platform::ErrorCode;
use libtock_platform::Syscalls;

use embedded_graphics_libtock::tock_screen::TockMonochromeScreen;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Circle, PrimitiveStyle};
use embedded_graphics::Drawable;

set_main! {main}
stack_size! {4000}

fn run() -> Result<(), ErrorCode> {
    let mut screen = TockMonochromeScreen::new();

    let width = screen.get_width();
    let height = screen.get_height();

    let button_count = Buttons::count()?;
    writeln!(Console::writer(), "[BUTTONS] Count: {}", button_count).unwrap();

    // Calculate where the buttons should be drawn.
    let button_padding_px = (button_count - 1) * 2;
    let max_x = (width - button_padding_px) / button_count;
    let max_y = height - 2;
    let diameter = core::cmp::min(max_x, max_y);
    let buttons_width = (diameter * button_count) + button_padding_px;
    let padding_left_px = (width - buttons_width) / 2;
    let y = (height / 2) - (diameter / 2);

    // Draw the initial button outlines.
    for i in 0..button_count {
        let x = padding_left_px + ((diameter + 2) * i);

        let _ = Circle::new(Point::new(x as i32, y as i32), diameter)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut screen);
    }
    let _ = screen.flush();

    // Now wait for button presses. Record what happened in the upcall.
    let buttons: [Cell<ButtonState>; 10] = [const { Cell::new(ButtonState::Released) }; 10];
    let changed: Cell<bool> = Cell::new(false);

    let listener = ButtonListener(|button, state| {
        // If the button state changed, record it.
        if buttons[button as usize].get() != state {
            buttons[button as usize].set(state);
            changed.set(true);
        }
    });
    share::scope(|subscribe| {
        // Subscribe to the button callback.
        Buttons::register_listener(&listener, subscribe).unwrap();

        // Enable interrupts for each button press.
        for i in 0..button_count {
            Buttons::enable_interrupts(i).unwrap();
        }

        // Wait for buttons to be pressed.
        loop {
            TockSyscalls::yield_wait();

            // If a button state changed, re-draw the buttons.
            if changed.get() {
                changed.set(false);

                let mut screen = TockMonochromeScreen::new();

                // Draw Circles
                for i in 0..button_count {
                    let x = padding_left_px + ((diameter + 2) * i);

                    // Draw outer circle
                    let _ = Circle::new(Point::new(x as i32, y as i32), diameter)
                        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                        .draw(&mut screen);

                    match buttons[i as usize].get() {
                        ButtonState::Pressed => {
                            let _ =
                                Circle::new(Point::new(x as i32 + 1, y as i32 + 1), diameter - 2)
                                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                                    .draw(&mut screen);
                        }
                        ButtonState::Released => {
                            let _ =
                                Circle::new(Point::new(x as i32 + 1, y as i32 + 1), diameter - 2)
                                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                                    .draw(&mut screen);
                        }
                    }
                }
                let _ = screen.flush();
            }
        }
    });

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(_e) => {
            writeln!(Console::writer(), "[BUTTONS] Err could not run app").unwrap();
        }
    }
}
