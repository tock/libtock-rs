#![feature(generator_trait)]
#![no_std]

pub use core::*;

pub mod future {
    pub use core::future::Future;
    use core::ops::Generator;
    use core::pin::Pin;
    use core::task::Poll;

    pub fn poll_with_tls_context<F>(f: Pin<&mut F>) -> Poll<F::Output>
    where
        F: Future,
    {
        crate::executor::poll(f)
    }

    pub fn from_generator<G: Generator<Yield = ()>>(
        generator: G,
    ) -> impl Future<Output = G::Return> {
        crate::executor::from_generator(generator)
    }
}

pub mod executor {
    use core::future::Future;
    use core::ops::Generator;
    use core::ops::GeneratorState;
    use core::pin::Pin;
    use core::ptr;
    use core::task::Context;
    use core::task::Poll;
    use core::task::RawWaker;
    use core::task::RawWakerVTable;
    use core::task::Waker;

    const DUMMY_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(get_waker, do_nothing, do_nothing, do_nothing);
    const DUMMY_WAKER: RawWaker = RawWaker::new(ptr::null(), &DUMMY_WAKER_VTABLE);

    extern "Rust" {
        #[link_name = "libtock::syscalls::yieldk"]
        fn yieldk();
    }

    pub fn block_on<T>(mut future: impl Future<Output = T>) -> T {
        let waker = unsafe { Waker::from_raw(DUMMY_WAKER) };
        let mut context = Context::from_waker(&waker);

        loop {
            let pinned_future = unsafe { Pin::new_unchecked(&mut future) };
            let result = pinned_future.poll(&mut context);
            match result {
                Poll::Pending => unsafe { yieldk() },
                Poll::Ready(value) => {
                    return value;
                }
            }
        }
    }

    pub(crate) fn poll<F: Future>(pinned_future: Pin<&mut F>) -> Poll<F::Output> {
        let waker = unsafe { Waker::from_raw(DUMMY_WAKER) };
        let mut context = Context::from_waker(&waker);
        pinned_future.poll(&mut context)
    }

    pub(crate) fn from_generator<G: Generator<Yield = ()>>(
        generator: G,
    ) -> impl Future<Output = G::Return> {
        GeneratorFuture { generator }
    }

    struct GeneratorFuture<G> {
        generator: G,
    }

    impl<G: Generator<Yield = ()>> Future for GeneratorFuture<G> {
        type Output = G::Return;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let pin = unsafe { Pin::new_unchecked(&mut Pin::into_inner_unchecked(self).generator) };
            match pin.resume() {
                GeneratorState::Yielded(()) => Poll::Pending,
                GeneratorState::Complete(out) => Poll::Ready(out),
            }
        }
    }

    const fn get_waker(_x: *const ()) -> RawWaker {
        DUMMY_WAKER
    }

    const fn do_nothing(_x: *const ()) {}
}
