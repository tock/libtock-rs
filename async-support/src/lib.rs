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

    extern "Rust" {
        #[link_name = "libtock::syscalls::raw::yieldk"]
        fn yieldk();
    }

    /// # Safety
    ///
    /// [[block_on]] yields whenever a future cannot make any progress at present. Yielding is considered unsafe.
    pub unsafe fn block_on<T>(mut future: impl Future<Output = T>) -> T {
        // Contract described in the Rustdoc: "A value, once pinned, must remain pinned forever (...).".
        // IOW calling Pin::new_unchecked is safe as long as no &mut future is leaked after pinning.
        let mut pinned_future = Pin::new_unchecked(&mut future);

        loop {
            match poll(pinned_future.as_mut()) {
                Poll::Pending => yieldk(),
                Poll::Ready(value) => {
                    return value;
                }
            }
        }
    }

    pub(crate) fn poll<F: Future>(pinned_future: Pin<&mut F>) -> Poll<F::Output> {
        let waker = unsafe { Waker::from_raw(get_dummy_waker()) };
        let mut context = Context::from_waker(&waker);
        pinned_future.poll(&mut context)
    }

    // Since Tock OS comes with waking-up functionality built-in, we use dummy wakers that do nothing at all.
    fn get_dummy_waker() -> RawWaker {
        fn clone(_x: *const ()) -> RawWaker {
            get_dummy_waker()
        }

        fn do_nothing(_x: *const ()) {}

        // This vtable implements the methods required for managing the lifecycle of the wakers.
        // Our wakers are dummies, so those functions don't do anything.
        static DUMMY_WAKER_VTABLE: RawWakerVTable =
            RawWakerVTable::new(clone, do_nothing, do_nothing, do_nothing);

        // The wakers don't have any implementation, so the instance can simply be null.
        RawWaker::new(ptr::null(), &DUMMY_WAKER_VTABLE)
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
            // Pin::map_unchecked_mut is safe as long as the move and drop guarantees are propagated through the mapping.
            // This is trivially satisfied since our future is only a newtype decorator of the generator.
            let pinned_generator =
                unsafe { self.map_unchecked_mut(|future| &mut future.generator) };

            match pinned_generator.resume(()) {
                GeneratorState::Yielded(()) => Poll::Pending,
                GeneratorState::Complete(out) => Poll::Ready(out),
            }
        }
    }
}
