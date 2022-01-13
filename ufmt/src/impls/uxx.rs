use core::str;

use crate::{uDebug, uDisplay, uWrite, Formatter};

macro_rules! uxx_hex {
    ($n:expr, $buf:expr, $pretty:expr) => {{
        let mut n = $n;
        let mut i = $buf.len() - 1;
        loop {
            *$buf
                .get_mut(i)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = {
                let d = (n as u8) & 0xf;
                if d < 10 {
                    d + b'0'
                } else {
                    (d - 10) + b'a'
                }
            };
            n = n >> 4;

            if n == 0 {
                break;
            } else {
                i -= 1;
            }
        }

        if $pretty {
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

macro_rules! uxx {
    ($n:expr, $buf:expr) => {{
        let mut n = $n;
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

        unsafe { str::from_utf8_unchecked($buf.get(i..).unwrap_or_else(|| assume_unreachable!())) }
    }};
}

fn usize(n: usize, buf: &mut [u8]) -> &str {
    uxx!(n, buf)
}

fn usize_hex(n: usize, buf: &mut [u8], pretty: bool) -> &str {
    uxx_hex!(n, buf, pretty)
}

impl uDebug for u8 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            let mut buf: [u8; 4] = [0; 4];
            f.write_str(usize_hex(usize::from(*self), &mut buf, f.pretty))
        } else {
            let mut buf: [u8; 3] = [0; 3];
            f.write_str(usize(usize::from(*self), &mut buf))
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
        if f.hex {
            let mut buf: [u8; 6] = [0; 6];
            f.write_str(usize_hex(usize::from(*self), &mut buf, f.pretty))
        } else {
            let mut buf: [u8; 5] = [0; 5];
            f.write_str(usize(usize::from(*self), &mut buf))
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
        let mut buf: [u8; 10] = [0; 10];
        if f.hex {
            f.write_str(usize_hex(*self as usize, &mut buf, f.pretty))
        } else {
            f.write_str(usize(*self as usize, &mut buf))
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
        if f.hex {
            let mut buf: [u8; 18] = [0; 18];

            let s = uxx_hex!(*self, buf, f.pretty);
            f.write_str(s)
        } else {
            let mut buf: [u8; 20] = [0; 20];

            let s = uxx!(*self, buf);
            f.write_str(s)
        }
    }

    #[cfg(target_pointer_width = "64")]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            let mut buf: [u8; 18] = unsafe { crate::uninitialized() };

            f.write_str(usize_hex(*self as usize, &mut buf, f.pretty))
        } else {
            let mut buf: [u8; 20] = [0; 20];

            f.write_str(usize(*self as usize, &mut buf))
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
        if f.hex {
            let mut buf: [u8; 34] = [0; 34];

            let s = uxx_hex!(*self, buf, f.pretty);
            f.write_str(s)
        } else {
            let mut buf: [u8; 39] = [0; 39];

            let s = uxx!(*self, buf);
            f.write_str(s)
        }
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
