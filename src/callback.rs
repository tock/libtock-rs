pub trait SubscribableCallback {
    fn driver_number(&self) -> usize;

    fn subscribe_number(&self) -> usize;

    fn call_rust(&mut self, arg0: usize, arg1: usize, arg2: usize);
}

pub struct CallbackSubscription<CB: SubscribableCallback> {
    #[allow(dead_code)] // Used in drop
    pub(crate) callback: CB,
}
