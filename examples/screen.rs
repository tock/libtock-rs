//! Screen API example.
//! Prints screen properties and displays patterns in corners:
//! top-left: black, top-right: white, bottom-left: vertical stripes

#![no_main]
#![no_std]

use core::cmp;
use core::fmt::Write;
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};
use libtock::screen::{lightness, Grid, PixelStreamFormat, Rectangle, Screen};

set_main! {main}
stack_size! {0x4000}

const BUFSIZE: usize = 16 * 16 * 32; // 8192 bytes max

fn clip_to_grid(r: Rectangle, grid: Grid) -> Rectangle {
    let floor = |i: u16, r| i - i % r;
    let ceil = |i, r| if i % r != 0 { i - i % r + r } else { i };
    Rectangle {
        x: grid.x_offset + ceil(r.x + grid.width - grid.x_offset, grid.width) - grid.width,
        y: grid.y_offset + ceil(r.y + grid.height - grid.y_offset, grid.height) - grid.height,
        width: grid.x_offset + floor(r.x + r.width - grid.x_offset, grid.width) - r.x,
        height: grid.y_offset + floor(r.y + r.height - grid.y_offset, grid.height) - r.y,
    }
}

fn main() {
    Screen::set_brightness(lightness::MAX).unwrap();
    Screen::set_power(true).unwrap();

    writeln!(Console::writer(), "Current resolution:").unwrap();
    let res = Screen::get_resolution().unwrap();
    writeln!(Console::writer(), "{:?}", res).unwrap();
    writeln!(Console::writer(), "Pixel format:").unwrap();
    let (px, grid) = Screen::get_pixel_format().unwrap();
    writeln!(Console::writer(), "{:?}", px).unwrap();
    writeln!(Console::writer(), "Grid:").unwrap();
    writeln!(Console::writer(), "{:?}", grid).unwrap();

    // If width or height is odd, leave a stripe in the middle
    let halfwidth = res.width as u16 / 2;
    let halfheight = res.height as u16 / 2;

    // black top left
    {
        let buf = [0xff; BUFSIZE];
        let frame = clip_to_grid(
            Rectangle {
                x: 0,
                y: 0,
                width: 16,  // halfwidth,
                height: 16, //halfheight,
            },
            grid,
        );
        Screen::set_frame(frame).unwrap();
        Screen::write(&buf).unwrap();
    }
    // white top right
    {
        let buf = [0xff; BUFSIZE];
        let rec = Rectangle {
            x: res.width as u16 - halfwidth,
            y: 0,
            width: 16,  //halfwidth,
            height: 16, //halfheight,
        };
        let frame = clip_to_grid(rec, grid);
        Screen::set_frame(frame).unwrap();
        Screen::write(&buf).unwrap();
    }
    // striped bottom left
    if false {
        let mut buf = [0x0; BUFSIZE];

        let window_size = cmp::max(px.get_bits_per_pixel() as usize, 8) / 8;
        let (even, odd) = if let PixelStreamFormat::Mono_1H8 = px {
            ([0b10101010; 1].as_slice(), [0b10101010; 1].as_slice())
        } else {
            ([0; 4].as_slice(), [0xff; 4].as_slice())
        };

        for (i, chunk) in buf.chunks_mut(window_size).enumerate() {
            let pattern = if i % 2 == 0 { even } else { odd };
            chunk.copy_from_slice(&pattern[..window_size]);
        }
        // This will cause checkerboard if half screen width is odd

        let frame = clip_to_grid(
            Rectangle {
                x: 0,
                y: res.height as u16 - halfheight,
                width: halfwidth,
                height: halfheight,
            },
            grid,
        );
        Screen::set_frame(frame).unwrap();
        Screen::write(&buf).unwrap();
    }
    // checkerboard first 2×2 tiles in top-left
    {
        let xcount = cmp::min((grid.x_offset as u32 + res.width) / grid.width as u32, 3) as u16;
        let ycount = cmp::min((grid.y_offset as u32 + res.height) / grid.height as u32, 3) as u16;
        let frame = Rectangle {
            x: grid.x_offset,
            y: grid.y_offset,
            width: xcount * grid.width,
            height: ycount * grid.height,
        };

        Screen::set_frame(frame).unwrap();
        let mut buf = [0x0; BUFSIZE];
        for col in 0..xcount {
            for row in 0..ycount {
                let bufpos = row * xcount + col;
                match px.get_bits_per_pixel() {
                    1 => {
                        let color = if (col ^ row) & 0b1 == 0b0 { 0x0 } else { 0xff };
                        writeln!(Console::writer(), "{} {} {} {:?}", col, row, bufpos, color)
                            .unwrap();
                        let width = grid.width as usize * 1 / 8;
                        let bufpos = bufpos as usize * width;
                        for i in 0..width {
                            buf[bufpos..][i] = color;
                        }
                    }
                    _ => {}
                }
            }
        }
        writeln!(Console::writer(), "{:?}", &buf[0..9]).unwrap();
        Screen::write(&buf).unwrap();
    }
}
