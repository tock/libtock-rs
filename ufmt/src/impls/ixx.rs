use core::str;

use crate::{uDebug, uDisplay, uWrite, Formatter};

macro_rules! ixx_pad {
    ($uxx:ty, $n:expr, $w:expr, $p:expr, $buf:expr) => {{
        let n = $n;
        let len = $buf.len();
        let negative = n.is_negative();
        let mut n = if negative {
            match n.checked_abs() {
                Some(n) => n as $uxx,
                None => <$uxx>::max_value() / 2 + 1,
            }
        } else {
            n as $uxx
        };
        let mut i = $buf.len() - 1;
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
            (Some(mut w), pad) => {
                // for space padding, pad before negative sign
                // for 0 padding, pad after negative sign
                if negative && pad == ' ' {
                    i -= 1;
                    *$buf
                        .get_mut(i)
                        .unwrap_or_else(|| unsafe { assume_unreachable!() }) = b'-';
                } else if negative {
                    w -= 1;
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

        if negative && ($w.is_none() || $p != ' ') {
            i -= 1;
            *$buf
                .get_mut(i)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = b'-';
        }

        unsafe { str::from_utf8_unchecked($buf.get(i..).unwrap_or_else(|| assume_unreachable!())) }
    }};
}

fn isize_pad(n: isize, width: Option<u8>, pad: char, buf: &mut [u8]) -> &str {
    ixx_pad!(usize, n, width, pad, buf)
}

impl uDebug for i8 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex.is_some() {
            <u8 as uDebug>::fmt(&(*self as u8), f)
        } else {
            let mut buf: [u8; 18] = [0; 18];

            f.write_str(isize_pad(isize::from(*self), f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for i8 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i8 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i16 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex.is_some() {
            <u16 as uDebug>::fmt(&(*self as u16), f)
        } else {
            let mut buf: [u8; 18] = [0; 18];

            f.write_str(isize_pad(isize::from(*self), f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for i16 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i16 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i32 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex.is_some() {
            <u32 as uDebug>::fmt(&(*self as u32), f)
        } else {
            let mut buf: [u8; 21] = [0; 21];

            f.write_str(isize_pad(*self as isize, f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for i32 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i32 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i64 {
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex.is_some() {
            <u64 as uDebug>::fmt(&(*self as u64), f)
        } else {
            let mut buf: [u8; 30] = [0; 30];

            let s = ixx_pad!(u64, *self, f.width, f.pad, buf);
            f.write_str(s)
        }
    }

    #[cfg(target_pointer_width = "64")]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex.is_some() {
            <u64 as uDebug>::fmt(&(*self as u64), f)
        } else {
            let mut buf: [u8; 30] = [0; 30];

            f.write_str(isize_pad(*self as isize, f.width, f.pad, &mut buf))
        }
    }
}

impl uDisplay for i64 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i64 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i128 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex.is_some() {
            <u128 as uDebug>::fmt(&(*self as u128), f)
        } else {
            let mut buf: [u8; 50] = [0; 50];

            let s = ixx_pad!(u128, *self, f.width, f.pad, buf);
            f.write_str(s)
        }
    }
}

impl uDisplay for i128 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i128 as uDebug>::fmt(self, f)
    }
}

impl uDebug for isize {
    #[cfg(target_pointer_width = "16")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i16 as uDebug>::fmt(&(*self as i16), f)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i32 as uDebug>::fmt(&(*self as i32), f)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i64 as uDebug>::fmt(&(*self as i64), f)
    }
}

impl uDisplay for isize {
    #[cfg(target_pointer_width = "16")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i16 as uDisplay>::fmt(&(*self as i16), f)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i32 as uDisplay>::fmt(&(*self as i32), f)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i64 as uDisplay>::fmt(&(*self as i64), f)
    }
}
