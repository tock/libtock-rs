#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
use core::cell::Cell;
use core::fmt::{Debug, Error, Formatter};
use core::time::Duration;
use ctap2_authenticator::credentials::{RpCredential, UserCredential};
use ctap2_authenticator::usbhid::{
    CtapHidCapabilities, CtapHidPlatform, KeepaliveResponse, TransactionProcessor,
};
use ctap2_authenticator::Authenticator;
use ctap2_authenticator::{
    AuthenticatorPlatform, CredentialDescriptorList, CtapOptions, PublicKey, Signature,
};
use libtock::ctap::{CtapRecvBuffer, CtapSendBuffer};
use libtock::println;
use libtock::result::TockResult;
use libtock::syscalls;

/// This is the provided implementation of `CtapHidPlatform` to be used in the `UsbContext`
pub(crate) struct UsbKeyHidPlatform {}

impl UsbKeyHidPlatform {
    pub fn new() -> Self {
        Self {}
    }
}

impl CtapHidPlatform for UsbKeyHidPlatform {
    // Should be kept in sync with the Crates version if possible
    const MAJOR_VERSION: u8 = 0;
    const MINOR_VERSION: u8 = 0;
    const BUILD_VERSION: u8 = 0;
    const CAPABILITIES: CtapHidCapabilities = CtapHidCapabilities {
        wink: true,
        cbor: true,
        msg: false,
    };

    fn wink(&mut self) {}

    fn cancel(&mut self) {
        println!("cancel");
        unimplemented!()
    }

    fn keepalive_needed(&mut self) -> KeepaliveResponse {
        println!("keepalive_needed");
        unimplemented!()
    }

    fn start_timer(&mut self) {
        println!("start_timer");
    }

    fn has_timed_out(&mut self) -> bool {
        println!("has_timed_out");
        false
    }
}

#[derive(Clone, Copy)]
pub struct HmacKeyCredential(pub(crate) [u8; 40]);

impl HmacKeyCredential {
    fn new(mac: &[u8; 32], nonce: [u8; 8]) -> Self {
        let mut new = [0; 40];
        new[..32].copy_from_slice(&mac[..]);
        new[32..].copy_from_slice(&nonce[..]);
        Self(new)
    }

    fn get_mac(&self) -> [u8; 32] {
        let mut mac = [0; 32];
        mac.copy_from_slice(&self.0[..32]);
        mac
    }
}

impl AsRef<[u8]> for HmacKeyCredential {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Debug for HmacKeyCredential {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "HmacKeyCredential {{ mac: {:?}, nonce: {:?} }}",
            &self.0[..32],
            &self.0[32..]
        )
    }
}

impl PartialEq for HmacKeyCredential {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl Eq for HmacKeyCredential {}

const ITERATOR_MAX_LENGTH: usize = 8;

pub struct HmacCredentialQueue {
    index: usize,
    length: usize,
    list: [Option<(HmacKeyCredential, u32)>; ITERATOR_MAX_LENGTH],
}

impl HmacCredentialQueue {
    pub(crate) fn new() -> Self {
        Self {
            index: 0,
            length: 0,
            list: [None; ITERATOR_MAX_LENGTH],
        }
    }

    pub(crate) fn push(&mut self, credential: HmacKeyCredential, counter: u32) -> Result<(), ()> {
        if self.length < ITERATOR_MAX_LENGTH {
            self.list[self.length] = Some((credential, counter));
            self.length += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.length
    }
}

impl Iterator for HmacCredentialQueue {
    type Item = (HmacKeyCredential, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.list[self.index].is_some() {
            self.index += 1;
            self.list[self.index - 1]
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct CtapPlatform;
impl AuthenticatorPlatform for CtapPlatform {
    const AAGUID: [u8; 16] = [
        0xf8, 0xa0, 0x11, 0xf3, 0x8c, 0x0a, 0x4d, 0x15, 0x80, 0x06, 0x17, 0x11, 0x1f, 0x9e, 0xdc,
        0x7d,
    ];
    const CERTIFICATE: Option<&'static [u8]> = None;
    const MAX_MSG_LENGTH: u16 = 7609;
    const SUPPORTED_OPTIONS: CtapOptions = CtapOptions {
        plat: None,
        rk: Some(true),
        client_pin: None,
        up: Some(true),
        uv: Some(false),
    };
    type CredentialId = HmacKeyCredential;
    type PublicKeyBuffer = [u8; 32];
    type SignatureBuffer = [u8; 32];
    type CredentialIterator = HmacCredentialQueue;

    fn reset(&mut self) -> Result<(), ()> {
        unimplemented!()
    }

    fn check_exclude_list(&mut self, _rp_id: &str, _list: CredentialDescriptorList) -> bool {
        unimplemented!()
    }

    fn locate_credentials(
        &mut self,
        _rp_id: &str,
        list: Option<CredentialDescriptorList>,
    ) -> (u16, Self::CredentialIterator) {
        match list {
            None => (0, HmacCredentialQueue::new()),
            Some(_list) => {
                let result = HmacCredentialQueue::new();

                // TODO Add location

                (result.len() as u16, result)
            }
        }
    }

    fn create_credential(
        &mut self,
        _rp: RpCredential<&[u8], &str>,
        _user: UserCredential<&[u8], &str>,
    ) -> (Self::CredentialId, PublicKey<Self::PublicKeyBuffer>, u32) {
        unimplemented!()
    }

    fn attest(
        &mut self,
        _id: &Self::CredentialId,
        _data: &[u8],
    ) -> Option<Signature<Self::SignatureBuffer>> {
        unimplemented!();
    }

    fn sign(
        &mut self,
        _id: &Self::CredentialId,
        _data: &[u8],
    ) -> Option<Signature<Self::SignatureBuffer>> {
        unimplemented!();
    }

    fn start_timeout(&mut self) {
        unimplemented!()
    }

    fn has_timed_out(&mut self, _time: Duration) -> bool {
        unimplemented!()
    }
}

#[alloc_error_handler]
unsafe fn alloc_error_handler(_: Layout) -> ! {
    println!("alloc_error_handler called");
    loop {
        syscalls::raw::yieldk();
    }
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut tp: Cell<TransactionProcessor<&'static mut [u8], UsbKeyHidPlatform>>;
    let mut temp_buffer = [0; libtock::ctap::RECV_BUFFER_SIZE];
    let mut drivers = libtock::retrieve_drivers()?;
    drivers.console.create_console();

    println!("Starting CTAP feature example");

    let ctap_driver = drivers.ctap.init_driver()?;

    let mut recv_buffer = CtapRecvBuffer::default();
    let recv_buffer = ctap_driver.init_recv_buffer(&mut recv_buffer)?;

    let mut send_buffer = CtapSendBuffer::default();
    let mut send_buffer = ctap_driver.init_send_buffer(&mut send_buffer)?;

    static mut BUFFER: &'static mut [u8] = &mut [0; 2048];
    unsafe {
        tp = Cell::new(TransactionProcessor::new(BUFFER, UsbKeyHidPlatform::new()));
    }

    let mut authenticator = Authenticator::create(CtapPlatform);

    let mut callback = |sent, _| {
        let mut looping = true;
        recv_buffer.read_bytes(&mut temp_buffer[..]);

        let temp_tp = tp.get_mut();
        let mut poll_msg = if sent == 0 { Some(&temp_buffer) } else { None };

        while looping {
            looping = false;
            let (maybe_request, maybe_data) = temp_tp.poll(poll_msg);

            if let Some(request) = maybe_request {
                let response = authenticator.process(request);
                temp_tp.response(response);
                looping = true;
            }

            if let Some(mut data) = maybe_data {
                send_buffer.write_bytes(&mut data[0..]);
                let _ = ctap_driver.send_data();
            }

            poll_msg = None;
        }
        let _ = ctap_driver.allow_receive();
    };

    let _subscription = ctap_driver.subscribe(&mut callback)?;
    ctap_driver.allow_receive()?;

    loop {
        unsafe { syscalls::raw::yieldk() };
    }
}
