use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

// TODO: Consider using FnMut
pub async fn wait_until<F: Fn() -> bool>(condition: F) {
    wait_for_value(move || if condition() { Some(()) } else { None }).await
}

pub async fn wait_for_value<T, F: Fn() -> Option<T>>(value_provider: F) -> T {
    WaitForValue { value_provider }.await
}

struct WaitForValue<F> {
    value_provider: F,
}

impl<T, F: Fn() -> Option<T>> Future for WaitForValue<F> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = (self.value_provider)() {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

/// Generates a synchronous `main()` function that calls the provided
/// asynchronous function. To avoid name conflicts, the asynchronous function
/// must not be called `main()`; `async_main()` is fine.
///
/// Example:
///     libtock::async_main!(async_main);
///     async fn async_main() {
///         // Snipped
///     }
#[macro_export]
macro_rules! async_main {
    ($main_name:ident) => {
        fn main() {
            use ::libtock::lang_items::Termination;
            static mut MAIN_INVOKED: bool = false;
            unsafe {
                // core::executor::block_on is unsafe and documented as being
                // unsafe to call from within a subscription callback.
                // Unfortunately, any code can call main(), so main() has to be
                // reentrant. To make this safe, we need to detect when main()
                // is called reentrantly and panic.
                if MAIN_INVOKED {
                    panic!("Main called recursively; this is unsafe with async_main!()");
                }
                MAIN_INVOKED = true;

                ::core::executor::block_on($main_name()).report();
            }
        }
    };
}

#[cfg(test)]
mod test {
    extern crate std;

    /// Test case verifying async_main!'s operation with a well-behaved
    /// (non-reentrant) async_main().
    #[test]
    fn async_main_good() {
        use std::sync::atomic::AtomicUsize;

        // Tracks the number of times increment() has been called. This is an
        // atomic rather than a Mutex as atomics can be const-initialized.
        // Accessed using SeqCst as that is the strongest,
        // easiest-to-reason-about form of consistency and this test doesn't
        // need to be particularly fast.
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        async fn increment() {
            COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        async_main!(increment);
        main();
        assert_eq!(COUNT.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    /// Test case verifying async_main! panics if invoked recursively rather
    /// than triggering UB.
    #[test]
    #[should_panic]
    fn async_main_reentrance() {
        // async_main function that calls back into main.
        async fn async_main() {
            main();
        }

        async_main!(async_main);
        main();
    }
}
