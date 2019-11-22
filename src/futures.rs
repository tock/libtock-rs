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
