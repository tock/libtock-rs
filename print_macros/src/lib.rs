#![no_std]

/*! Runtime debugging tools.

This is a low-level crate intended to use from libtock.
If you want to use the macros in a crate that is not part of libtock,
import `libtock::print_macros` instead.

When using within libtock, add `libtock_console` dependency.
*/

#[macro_export]
/// Prints the expression to the console, does not add a newline
macro_rules! print {
    ($($arg:tt)*) => ({
        use libtock_runtime::TockSyscalls;
        use core::fmt::Write;
        type Console = libtock_console::Console<TockSyscalls>;
        write!(&mut Console::writer(), $($arg)*).unwrap()
    });
}

#[macro_export]
/// Prints the expression to the console, ends with a newline
macro_rules! println {
    ($($arg:tt)*) => ({
        use libtock_runtime::TockSyscalls;
        use core::fmt::Write;
        type Console = libtock_console::Console<TockSyscalls>;
        writeln!(&mut Console::writer(), $($arg)*).unwrap()
    });
}

// Taken from rustc.
#[macro_export]
/// Prints out the expression to the console,
/// including file name and line number,
/// and then returns it.
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[cfg(test)]
mod test {
    use super::*;
    /// Checks if the macros compile without libtock in scope.
    pub fn test() {
        panic!("This is not supposed to be actually called");

        #[derive(Debug)]
        struct HelloWorld;

        dbg!(HelloWorld);
        let i: u32 = dbg!(0xdeadc0deu32);
        print!("Hello");
        println!(" again! {}", i);
    }
}
