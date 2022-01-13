use core::str;

use crate::{uDebug, uDisplay, uWrite, Formatter};

macro_rules! uxx_hex_pad {
    ($n:expr, $w:expr, $p:expr, $buf:expr, $pretty:expr, $lower:expr) => {{
        let mut n = $n;
        let len = $buf.len();
        let mut i = len - 1;
        loop {
            *$buf
                .get_mut(i)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = {
                let d = (n as u8) & 0xf;
                if d < 10 {
                    d + b'0'
                } else {
                    if $lower {
                        (d - 10) + b'a'
                    } else {
                        (d - 10) + b'A'
                    }
                }
            };
            n = n >> 4;

            if n == 0 {
                break;
            } else {
                i -= 1;
            }
        }

        // Now fill in padding up to the prescribed width.
        // We do not support widths shorter than the value being
        // printed, like core::fmt::format!()
        // Also, we currently only support ' ' and '0' padding.
        match ($w, $p) {
            // For now, we default to left padding for all int-like values
            (Some(mut w), pad) => {
                // for space padding, pad before 0x. For 0 padding,
                // pad after 0x
                if $pretty && pad == ' ' {
                    i -= 2;
                    *$buf
                        .get_mut(i + 1)
                        .unwrap_or_else(|| unsafe { assume_unreachable!() }) = b'x';
                    *$buf
                        .get_mut(i)
                        .unwrap_or_else(|| unsafe { assume_unreachable!() }) = b'0';
                } else if $pretty {
                    w -= 2;
                }
                while i > (len - (w as usize)) {
                    i -= 1;
                    let byte = match pad {
                        '0' => b'0',
                        _ => b' ',
                    };

                    *$buf
                        .get_mut(i)
                        .unwrap_or_else(|| unsafe { assume_unreachable!() }) = byte;
                }
            }
            _ => {}
        }

        if $pretty && ($w.is_none() || $p != ' ') {
            i -= 2;
            *$buf
                .get_mut(i + 1)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = b'x';
            *$buf
                .get_mut(i)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = b'0';
        }

        unsafe { str::from_utf8_unchecked($buf.get(i..).unwrap_or_else(|| assume_unreachable!())) }
    }};
}

macro_rules! uxx_pad {
    ($n:expr, $w: expr, $p:expr, $buf:expr) => {{
        let mut n = $n;
        let len = $buf.len();
        let mut i = len - 1;
        loop {
            *$buf
                .get_mut(i)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = (n % 10) as u8 + b'0';
            n = n / 10;

            if n == 0 {
                break;
            } else {
                i -= 1;
            }
        }
        // Now fill in padding up to the prescribed width.
        // We do not support widths shorter than the value being
        // printed, like core::fmt::format!()
        // Also, we currently only support ' ' and '0' padding.
        match ($w, $p) {
            // For now, we default to left padding for all int-like values
            (Some(w), pad) => {
                while i > (len - (w as usize)) {
                    i -= 1;
                    let byte = match pad {
                        '0' => b'0',
                        _ => b' ',
                    };

                    *$buf
                        .get_mut(i)
                        .unwrap_or_else(|| unsafe { assume_unreachable!() }) = byte;
                }
            }

            _ => {}
        }

        unsafe { str::from_utf8_unchecked($buf.get(i..).unwrap_or_else(|| assume_unreachable!())) }
    }};
}

fn usize_pad(n: usize, width: Option<u8>, pad: char, buf: &mut [u8]) -> &str {
    uxx_pad!(n, width, pad, buf)
}

fn usize_hex_pad(
    n: usize,
    width: Option<u8>,
    pad: char,
    buf: &mut [u8],
    pretty: bool,
    lower: bool,
) -> &str {
    uxx_hex_pad!(n, width, pad, buf, pretty, lower)
}

impl uDebug for u8 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut buf: [u8; 18] = [0; 18];
        if let Some(lower) = f.hex {
            f.write_str(usize_hex_pad(
                usize::from(*self),
                f.width,
                f.pad,
                &mut buf,
                f.pretty,
                lower,
            ))
        } else {
            f.write_str(usize_pad(usize::from(*self), f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for u8 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u8 as uDebug>::fmt(self, f)
    }
}

impl uDebug for u16 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut buf: [u8; 18] = [0; 18];
        if let Some(lower) = f.hex {
            f.write_str(usize_hex_pad(
                usize::from(*self),
                f.width,
                f.pad,
                &mut buf,
                f.pretty,
                lower,
            ))
        } else {
            f.write_str(usize_pad(usize::from(*self), f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for u16 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u16 as uDebug>::fmt(self, f)
    }
}

impl uDebug for u32 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut buf: [u8; 20] = [0; 20];
        if let Some(lower) = f.hex {
            f.write_str(usize_hex_pad(
                *self as usize,
                f.width,
                f.pad,
                &mut buf,
                f.pretty,
                lower,
            ))
        } else {
            f.write_str(usize_pad(*self as usize, f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for u32 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u32 as uDebug>::fmt(self, f)
    }
}

impl uDebug for u64 {
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut buf: [u8; 28] = [0; 28];
        let s = if let Some(lower) = f.hex {
            uxx_hex_pad!(*self, f.width, f.pad, &mut buf, f.pretty, lower)
        } else {
            uxx_pad!(*self, f.width, f.pad, &mut buf)
        };
        f.write_str(s)
    }

    #[cfg(target_pointer_width = "64")]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut buf: [u8; 30] = [0; 30];
        if let Some(lower) = f.hex {
            f.write_str(usize_hex_pad(
                *self as usize,
                f.width,
                f.pad,
                &mut buf,
                f.pretty,
                lower,
            ))
        } else {
            f.write_str(usize_pad(*self as usize, f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for u64 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u64 as uDebug>::fmt(self, f)
    }
}

impl uDebug for u128 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut buf: [u8; 49] = [0; 49];
        let s = if let Some(lower) = f.hex {
            uxx_hex_pad!(*self, f.width, f.pad, buf, f.pretty, lower)
        } else {
            uxx_pad!(*self, f.width, f.pad, buf)
        };
        f.write_str(s)
    }
}

impl uDisplay for u128 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u128 as uDebug>::fmt(self, f)
    }
}

impl uDebug for usize {
    #[cfg(target_pointer_width = "16")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u16 as uDebug>::fmt(&(*self as u16), f)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u32 as uDebug>::fmt(&(*self as u32), f)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u64 as uDebug>::fmt(&(*self as u64), f)
    }
}

impl uDisplay for usize {
    #[cfg(target_pointer_width = "16")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u16 as uDisplay>::fmt(&(*self as u16), f)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u32 as uDisplay>::fmt(&(*self as u32), f)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <u64 as uDisplay>::fmt(&(*self as u64), f)
    }
}
