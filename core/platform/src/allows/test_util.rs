//! Contains testing utilities for libtock_platform::allowed.

// A value that can be placed in an Allowed that checks whether it has been
// dropped.
#[derive(Default)]
pub(crate) struct DropCheck<'f> {
    pub flag: Option<&'f core::cell::Cell<bool>>,
    pub value: usize,
}

impl<'f> Drop for DropCheck<'f> {
    fn drop(&mut self) {
        if let Some(flag) = self.flag {
            flag.set(true);
        }
    }
}

// Note: DropCheck cannot be safely used in a non-test context, as DropCheck
// does not satisfy AllowReadable's "every bit pattern is valid" requirement.
// However, in the tests we aren't sharing the buffer with a real Tock kernel,
// and instead we guarantee that only valid instances of DropCheck are loaded
// into the buffer.
unsafe impl<'f> crate::AllowReadable for DropCheck<'f> {}

// Verify that DropCheck works, as none of the tests require it to actually set
// the drop flag.
#[test]
fn drop_check() {
    let flag = core::cell::Cell::new(false);
    let drop_check = DropCheck {
        flag: Some(&flag),
        value: 0,
    };
    drop(drop_check);
    assert_eq!(flag.get(), true);
}
