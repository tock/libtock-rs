use libtock_platform::{Callback, CallbackContext, PlatformApi};

pub struct SyncAdapter<AsyncResponse, Platform: PlatformApi> {
    platform: Platform,
    response: core::cell::Cell<Option<AsyncResponse>>,
}

impl<AsyncResponse, Platform: PlatformApi> SyncAdapter<AsyncResponse, Platform> {
    pub fn new(platform: Platform) -> SyncAdapter<AsyncResponse, Platform> {
        SyncAdapter { platform, response: core::cell::Cell::new(None) }
    }

    pub fn wait(&self) -> AsyncResponse {
        loop {
            if let Some(response) = self.response.take() {
                return response;
            }
            self.platform.run_callback();
        }
    }
}

impl<AsyncResponse, Platform: PlatformApi> Callback<AsyncResponse> for &SyncAdapter<AsyncResponse, Platform> {
    fn callback(self, _context: CallbackContext, response: AsyncResponse) {
        self.response.set(Some(response));
    }
}
