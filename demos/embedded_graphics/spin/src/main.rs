#![no_main]
#![no_std]
use libtock::alarm::Alarm;
use libtock::alarm::Milliseconds;
use libtock::runtime::{set_main, stack_size};

use embedded_graphics_libtock::tock_screen::TockMonochromeScreen;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::Drawable;

set_main! {main}
stack_size! {4000}

fn main() {
    let mut screen = TockMonochromeScreen::new();

    let width = screen.get_width() as i32;
    let height = screen.get_height() as i32;

    let center_x = width / 2;
    let center_y = height / 2;
    let radius = if width < height {
        center_x - 1
    } else {
        center_y - 1
    };

    let mut rot: usize = 0;

    loop {
        let _ = screen.clear(BinaryColor::Off);

        let angle = (rot as f32 / 100.0) * (2.0 * core::f32::consts::PI);

        let x = (center_x as f32 + (radius as f32 * libm::cosf(angle))) as i32;
        let y = (center_y as f32 + (radius as f32 * libm::sinf(angle))) as i32;

        let _ = Line::new(Point::new(center_x, center_y), Point::new(x, y))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut screen);

        let _ = screen.flush();

        Alarm::sleep_for(Milliseconds(200)).unwrap();

        rot = (rot + 1) % 100;
    }
}
