//! This is a featured CTAP example
//! WARNING! This currently uses unsound crypto operations
//! This is only a demo and should not be used in real enviroments
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
use core::cell::Cell;
use core::convert::TryInto;
use core::fmt::{self, Debug, Error, Formatter};
use core::time::Duration;
use ctap2_authenticator::credentials::{RpCredential, UserCredential};
use ctap2_authenticator::usbhid::{
    CtapHidCapabilities, CtapHidPlatform, KeepaliveResponse, TransactionProcessor,
};
use ctap2_authenticator::Authenticator;
use ctap2_authenticator::{
    AuthenticatorPlatform, CredentialDescriptorList, CtapOptions, PublicKey, Signature,
};
use generic_array::GenericArray;
use libtock::ctap::{CtapRecvBuffer, CtapSendBuffer};
use libtock::hmac::{HmacDataBuffer, HmacDestBuffer, HmacDriverFactory, HmacKeyBuffer};
use libtock::println;
use libtock::result::TockResult;
use libtock::syscalls;
use p256::ecdsa::{signature::Signer, SigningKey};
use p256::elliptic_curve::ff::PrimeField;
use p256::{Scalar, SecretKey};
use subtle::{Choice, ConditionallySelectable};

libtock_core::stack_size! {0x4000}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct PrivateKey(Scalar);

impl ConditionallySelectable for PrivateKey {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(ConditionallySelectable::conditional_select(
            &a.0, &b.0, choice,
        ))
    }
}

impl Default for PrivateKey {
    fn default() -> Self {
        Self(Scalar::default())
    }
}

impl PrivateKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        Scalar::from_repr(GenericArray::clone_from_slice(bytes)).map(|s| Self(s))
    }
}

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

pub(crate) struct CtapPlatform {
    hmac: HmacDriverFactory,
}

impl CtapPlatform {
    fn new(hmac: HmacDriverFactory) -> Self {
        Self { hmac }
    }
}

impl fmt::Debug for CtapPlatform {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl CtapPlatform {
    fn generate_key_seed(
        &mut self,
        rp_id: &str,
        nonce: &[u8; 8],
    ) -> [u8; libtock::hmac::DEST_BUFFER_SIZE] {
        let hmac_driver;
        match self.hmac.init_driver() {
            Err(_) => {
                panic!("Hmac init error");
            }
            Ok(driver) => {
                hmac_driver = driver;
            }
        }

        let mut key_buffer = HmacKeyBuffer::default();
        for (i, d) in rp_id.as_bytes().iter().enumerate() {
            key_buffer[i] = *d;
        }
        // We need to make sure this isn't dropped straight away
        let key_buffer_ret = hmac_driver.init_key_buffer(&mut key_buffer);
        if key_buffer_ret.is_err() {
            panic!("Hmac key buffer init error");
        }

        let mut data_buffer = HmacDataBuffer::default();
        for (i, d) in nonce.iter().enumerate() {
            data_buffer[i] = *d;
        }
        // We need to make sure this isn't dropped straight away
        let data_buffer_ret = hmac_driver.init_data_buffer(&mut data_buffer);
        if data_buffer_ret.is_err() {
            panic!("Hmac data buffer init error");
        }

        let mut dest_buffer = HmacDestBuffer::default();
        let seed_buffer;
        match hmac_driver.init_dest_buffer(&mut dest_buffer) {
            Err(_) => {
                panic!("Hmac subscribe error");
            }
            Ok(buffer) => {
                seed_buffer = buffer;
            }
        }

        let mut callback = |_result, _digest| {};

        // We need to make sure this isn't dropped straight away
        let subscribe = hmac_driver.subscribe(&mut callback);
        if subscribe.is_err() {
            panic!("Hmac subscribe error");
        }

        if hmac_driver.run().is_err() {
            panic!("Hmac run error");
        }

        // Yield waiting for the HMAC callback
        unsafe {
            syscalls::raw::yieldk();
        }

        let mut temp_buffer = [0; libtock::hmac::DEST_BUFFER_SIZE];
        seed_buffer.read_bytes(&mut temp_buffer[..]);
        temp_buffer
    }

    /// Generates a Credential from a CredentialId and checks that the Id is valid
    fn checked_generate_hmac_cred(
        &mut self,
        rp_id: &str,
        credential_id: &[u8],
    ) -> Option<HmacKeyCredential> {
        if credential_id.len() != 40 {
            return None;
        }

        let received_mac = &credential_id[..32];

        let seed_buffer = self.generate_key_seed(rp_id, &credential_id[32..40].try_into().unwrap());

        let credential =
            HmacKeyCredential::new(&seed_buffer, credential_id[32..].try_into().unwrap());

        let generated_mac = &credential.0[..32];

        if generated_mac == received_mac {
            Some(credential)
        } else {
            None
        }
    }
}

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
        println!("reset");
        unimplemented!()
    }

    fn check_exclude_list(&mut self, _rp_id: &str, _list: CredentialDescriptorList) -> bool {
        println!("check_exclude_list");
        unimplemented!()
    }

    fn locate_credentials(
        &mut self,
        rp_id: &str,
        list: Option<CredentialDescriptorList>,
    ) -> (u16, Self::CredentialIterator) {
        match list {
            None => (0, HmacCredentialQueue::new()),
            Some(list) => {
                let mut result = HmacCredentialQueue::new();

                for descriptor in list {
                    match self.checked_generate_hmac_cred(rp_id, descriptor.get_id()) {
                        None => (),
                        Some(credential) => {
                            match result.push(credential, 0) {
                                Ok(()) => (),
                                // NOTE: We just truncate if we don't have enough space to store all
                                // fitting credentials
                                Err(()) => break,
                            }
                        }
                    }
                }

                (result.len() as u16, result)
            }
        }
    }

    fn create_credential(
        &mut self,
        rp: RpCredential<&[u8], &str>,
        _user: UserCredential<&[u8], &str>,
    ) -> (Self::CredentialId, PublicKey<Self::PublicKeyBuffer>, u32) {
        println!("create_credential");
        let rp_id = core::str::from_utf8(rp.rp_id()).unwrap();
        // This nonce is static!!!!
        // This is really bad cryptowise, let's print a warning
        // TODO: Convert this to generate a nonce from Tock's TRNG
        println!("WARNING!!! The nonce is static, this key is insecure");
        println!("WARNING!!! Do not use this anywhere important");
        let nonce = [0; 8];

        let seed_buffer = self.generate_key_seed(rp_id, &nonce);

        let credential = HmacKeyCredential::new(&seed_buffer, nonce.try_into().unwrap());

        let secret_key = SecretKey::from_bytes(&seed_buffer[0..32]).unwrap();

        let pub_key = p256::EncodedPoint::from_secret_key(&secret_key, false);

        let x: [u8; 32] = pub_key.as_bytes()[1..33].try_into().unwrap();
        let y: [u8; 32] = pub_key.as_bytes()[33..].try_into().unwrap();

        let ret_key = PublicKey::nistp256(x, y);

        (credential, ret_key, 0)
    }

    fn attest(
        &mut self,
        id: &Self::CredentialId,
        data: &[u8],
    ) -> Option<Signature<Self::SignatureBuffer>> {
        // TODO: Should attest be different then sign?
        let attest = {
            let secret_key = SecretKey::from_bytes(id.get_mac()).unwrap();

            let signer = SigningKey::from(&secret_key);

            let sig = signer.sign(data);

            let mut r: [u8; 32] = [0; 32];
            r.clone_from_slice(&sig.r().as_ref().to_bytes());
            let mut s: [u8; 32] = [0; 32];
            s.clone_from_slice(&sig.s().as_ref().to_bytes());

            Signature::nistp256(r, s)
        };

        Some(attest)
    }

    fn sign(
        &mut self,
        id: &Self::CredentialId,
        data: &[u8],
    ) -> Option<Signature<Self::SignatureBuffer>> {
        let attest = {
            let secret_key = SecretKey::from_bytes(id.get_mac()).unwrap();
            let signer = SigningKey::from(&secret_key);
            let sig = signer.sign(data);

            let mut r: [u8; 32] = [0; 32];
            r.clone_from_slice(&sig.r().as_ref().to_bytes());
            let mut s: [u8; 32] = [0; 32];
            s.clone_from_slice(&sig.s().as_ref().to_bytes());

            Signature::nistp256(r, s)
        };

        Some(attest)
    }

    fn start_timeout(&mut self) {
        println!("start_timeout");
        unimplemented!()
    }

    fn has_timed_out(&mut self, _time: Duration) -> bool {
        println!("has_timed_out");
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

    let mut authenticator = Authenticator::create(CtapPlatform::new(drivers.hmac));

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
