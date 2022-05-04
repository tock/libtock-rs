#![no_std]

use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub mod lightness {
    pub const OFF: u32 = 0;
    pub const MIN: u32 = 1;
    pub const MAX: u32 = 65536;
}

#[derive(Copy, Clone, Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum PixelStreamFormat {
    /// Pixels encoded as 1-bit, 8 consecutive bits (most to least significant)
    /// form 8 horizontal pixels left-to-right.
    Mono_1H8 = 0,
    /// Pixels encoded as 1-bit, 8 consecutive bits (most to least significant)
    /// form 8 vertical pixels top-to-bottom.
    /// Reaching end of line advances the cursor to the beginning of the line
    /// and 8 pixels downwards.
    Mono_1V8 = 1,
    /// A single byte carries two pixels horizontally: higher nibble on the left,
    /// lower on the right.
    /// Each color channel is 1 bit (starting with most significant): RGBX,
    /// where X is a padding bit.
    RGB_111xH2 = 2,
    /// Pixels encoded as 2-bit red channel, 3-bit green channel, 3-bit blue channel.
    RGB_233 = 3,
    /// Pixels encoded as 5-bit red channel, 6-bit green channel, 5-bit blue channel.
    RGB_565 = 4,
    /// Pixels encoded as 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    RGB_888 = 5,
    /// Pixels encoded as 8-bit alpha channel, 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    ARGB_8888 = 6,
    // other pixel formats may be defined.
}

impl PixelStreamFormat {
    /// Bits per pixel
    pub fn get_bits_per_pixel(&self) -> u8 {
        match self {
            Self::Mono_1H8 => 1,
            Self::Mono_1V8 => 1,
            Self::RGB_111xH2 => 4,
            Self::RGB_233 => 8,
            Self::RGB_565 => 16,
            Self::RGB_888 => 24,
            Self::ARGB_8888 => 32,
        }
    }
}

impl TryFrom<u32> for PixelStreamFormat {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, ()> {
        match v {
            0 => Ok(Self::Mono_1H8),
            1 => Ok(Self::Mono_1V8),
            2 => Ok(Self::RGB_111xH2),
            3 => Ok(Self::RGB_233),
            4 => Ok(Self::RGB_565),
            5 => Ok(Self::RGB_888),
            6 => Ok(Self::ARGB_8888),
            _ => Err(()),
        }
    }
}

/// Placement of a grid of rectangles on the display.
#[derive(Copy, Clone, Debug)]
pub struct Grid {
    /// Placement of a vertical edge of a rectangle
    /// relative to the left edge of the display.
    /// Additional rectangles are tiled to the left and right
    /// without overlapping, until they fill the entire display area.
    /// Greater values are further to the right.
    pub x_offset: u16,
    /// Placement of a horizontal edge of a rectangle,
    /// relative to top edge of the display.
    /// Greater values are further downwards.
    pub y_offset: u16,
    /// Width of a single rectangle.
    pub width: u16,
    /// Height of a single rectangle.
    pub height: u16,
}

/// The screen driver.
///
/// It allows libraries to control an attached display module.
///
/// # Example
/// ```ignore
/// use libtock2::Screen;
///
/// // Writes "foo", followed by a newline, to the console
/// let mut writer = Console::writer();
/// writeln!(writer, foo).unwrap();
/// ```
pub struct Screen<
    S: Syscalls,
    C: platform::allow_ro::Config + platform::subscribe::Config = DefaultConfig,
>(S, C);

impl<S: Syscalls, C: platform::allow_ro::Config + platform::subscribe::Config> Screen<S, C> {
    /// Run a check against the console capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn driver_check() -> bool {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).is_success()
    }

    pub fn get_resolution() -> Result<Resolution, ErrorCode> {
        S::command(DRIVER_NUM, command::RESOLUTION, 0, 0)
            .to_result()
            .map(|(width, height): (u32, u32)| Resolution { width, height })
    }

    pub fn get_pixel_format() -> Result<(PixelStreamFormat, Grid), ErrorCode> {
        S::command(DRIVER_NUM, command::PIXEL_FORMAT, 0, 0)
            .to_result()
            .and_then(|(pf, g): (u32, u64)| {
                PixelStreamFormat::try_from(pf)
                    .map(|pf| {
                        (
                            pf,
                            Grid {
                                width: (g >> 48) as u16,
                                height: (g >> 32) as u16,
                                x_offset: (g >> 16) as u16,
                                y_offset: g as u16,
                            },
                        )
                    })
                    .map_err(|()| ErrorCode::Invalid)
            })
    }

    pub fn set_power(on: bool) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope(|subscribe| {
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMMANDS }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::SET_POWER, on as u32, 0).to_result()?;

            loop {
                S::yield_wait();
                if let Some((_result, command)) = called.get() {
                    if command == Callback::Ready as u32 {
                        return Ok(());
                    }
                }
            }
        })
    }

    pub fn set_brightness(lightness: u32) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope(|subscribe| {
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMMANDS }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::SET_BRIGHTNESS, lightness, 0).to_result()?;

            loop {
                S::yield_wait();
                if let Some((_result, command)) = called.get() {
                    if command == Callback::Ready as u32 {
                        return Ok(());
                    }
                }
            }
        })
    }

    pub fn set_frame(frame: Rectangle) -> Result<(), ErrorCode> {
        let reg1 = (frame.x as u32) << 16 | frame.y as u32;
        let reg2 = (frame.width as u32) << 16 | frame.height as u32;
        S::command(DRIVER_NUM, command::SET_FRAME, reg1, reg2).to_result()
    }

    /// Writes the whole buffer of pixel data to selected frame.
    /// Does not check buffer alignment or bounds.
    pub fn write(buffer: &[u8]) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::COMMANDS }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::WRITE }>(allow_ro, buffer)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMMANDS }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::WRITE, buffer.len() as u32, 0).to_result()?;

            loop {
                S::yield_wait();
                if let Some((_result, command)) = called.get() {
                    if command == 0 {
                        //Callback::WriteComplete as u32 {
                        return Ok(());
                    }
                }
            }
        })
    }
}

// #[cfg(test)]
// mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x90001;

// Command IDs
#[allow(unused)]
mod command {
    pub const DRIVER_CHECK: u32 = 0;
    pub const SET_POWER: u32 = 2;
    pub const SET_BRIGHTNESS: u32 = 3;
    pub const RESOLUTION: u32 = 23;
    pub const PIXEL_FORMAT: u32 = 25;
    pub const SET_FRAME: u32 = 100;
    pub const WRITE: u32 = 200;
}

#[allow(unused)]
enum Callback {
    Ready = 0,
    WriteComplete = 1,
    CommandComplete = 2,
}

mod subscribe {
    pub const COMMANDS: u32 = 0;
}

mod allow_ro {
    pub const WRITE: u32 = 0;
}
