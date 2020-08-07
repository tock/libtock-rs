#[derive(Clone,Copy)]
pub struct Console<P> {
    platform: P,
}

impl<P> Console<P> {
    pub const fn new(platform: P) -> Console<P> {
        Console { platform }
    }
}

impl<P: libtock_platform::PlatformApi> Console<P> {
    pub fn check_exists(self) -> libtock_platform::ReturnCode {
        self.platform.command(1, 0, 0, 0)
    }
}
