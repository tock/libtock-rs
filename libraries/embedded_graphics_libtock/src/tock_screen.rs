use libtock::display::Display;
use libtock_platform::ErrorCode;

pub struct TockMonochromeScreen {
    /// The framebuffer for the max supported screen size (128x64). Each pixel
    /// is a bit.
    framebuffer: [u8; (128 * 64) / 8],
    width: u32,
    height: u32,
}

impl TockMonochromeScreen {
    pub fn new() -> Self {
        let (width, height) = Display::get_resolution().unwrap_or((0, 0));

        Self {
            framebuffer: [0; 1024],
            width,
            height,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Updates the display from the framebuffer.
    pub fn flush(&self) -> Result<(), ErrorCode> {
        Display::set_write_frame(0, 0, self.width, self.height)?;
        Display::write(&self.framebuffer)?;
        Ok(())
    }
}

impl embedded_graphics::draw_target::DrawTarget for TockMonochromeScreen {
    type Color = embedded_graphics::pixelcolor::BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for embedded_graphics::Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0
                && coord.x < self.width as i32
                && coord.y >= 0
                && coord.y < self.height as i32
            {
                const X_FACTOR: usize = 1;
                const Y_FACTOR: usize = 8;
                const X_COLS: usize = 128;

                let x = coord.x as usize;
                let y = coord.y as usize;

                let byte_index = (x / X_FACTOR) + ((y / Y_FACTOR) * X_COLS);
                let bit_index = y % Y_FACTOR;

                if color.is_on() {
                    self.framebuffer[byte_index] |= 1 << bit_index;
                } else {
                    self.framebuffer[byte_index] &= !(1 << bit_index);
                }
            }
        }

        Ok(())
    }
}

impl embedded_graphics::geometry::OriginDimensions for TockMonochromeScreen {
    fn size(&self) -> embedded_graphics::geometry::Size {
        embedded_graphics::geometry::Size::new(128, 64)
    }
}
