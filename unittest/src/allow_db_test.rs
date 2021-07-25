//! Unit test cases for functionality in allow_db.

use crate::allow_db::*;
use core::cell::Cell;

// Utility to call insert_ro_buffer with a slice.
// Safety: insert_ro_slice does not prevent RoAllowBuffer from outliving slice.
// Instead, the caller must make sure its use patterns don't cause invalid
// accesses.
unsafe fn insert_ro_slice(
    db: &mut AllowDb,
    slice: &[Cell<u8>],
) -> Result<RoAllowBuffer, OverlapError> {
    // Safety: The address and len arguments are derived directly from a slice,
    // and therefore satisfy insert_ro_buffer's precondition.
    unsafe { db.insert_ro_buffer(slice.as_ptr().into(), slice.len().into()) }
}

// Utility to call insert_rw_buffer with a slice.
// Safety: insert_ro_slice does not prevent RoAllowBuffer from outliving slice.
// Instead, the caller must make sure its use patterns don't cause invalid
// accesses.
unsafe fn insert_rw_slice(
    db: &mut AllowDb,
    slice: &[Cell<u8>],
) -> Result<RwAllowBuffer, OverlapError> {
    // Safety: The address and len arguments are derived directly frwm a slice,
    // and therefore satisfy insert_rw_buffer's precondition.
    unsafe { db.insert_rw_buffer(slice.as_ptr().into(), slice.len().into()) }
}

// Utility to return a RoAllowBuffer and verify the returned register values
// match the provided slice.
fn remove_ro_check(db: &mut AllowDb, buffer: RoAllowBuffer, slice: &[Cell<u8>]) {
    let (address, len) = db.remove_ro_buffer(buffer);
    let address: *const u8 = address.into();
    assert_eq!(address, slice.as_ptr() as *const u8);
    let len: usize = len.into();
    assert_eq!(len, slice.len());
}

// Utility to return a RwAllowBuffer and verify the returned register values
// match the prwvided slice.
fn remove_rw_check(db: &mut AllowDb, buffer: RwAllowBuffer, slice: &[Cell<u8>]) {
    let (address, len) = db.remove_rw_buffer(buffer);
    let address: *mut u8 = address.into();
    assert_eq!(address, slice.as_ptr() as *mut u8);
    let len: usize = len.into();
    assert_eq!(len, slice.len());
}

#[test]
fn allow_db() {
    let mut db: AllowDb = Default::default();
    let fake_memory: &mut [u8] = &mut [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let fake_memory = Cell::from_mut(fake_memory).as_slice_of_cells();
    // Safety: Big unsafe block because insert_ro_slice and insert_rw_slice do
    // not protect lifetimes. We have to return all slices before they become
    // invalid, which happens after this blocks ends (when we check
    // fake_memory's value).
    unsafe {
        let ro_buffer_2_5 = insert_ro_slice(&mut db, &fake_memory[2..=5]).unwrap();
        // A zero-sized buffer should not alias.
        let ro_buffer_3_empty = insert_ro_slice(&mut db, &fake_memory[3..3]).unwrap();
        let rw_buffer_3_empty = insert_rw_slice(&mut db, &fake_memory[3..3]).unwrap();
        // Generate an overlapping allow, which should return an error.
        insert_rw_slice(&mut db, &fake_memory[4..=7]).unwrap_err();
        // Generate a variety of overlaps: overlapping just the first byte, the last
        // byte, a larger range, a smaller range, and an identical range.
        insert_rw_slice(&mut db, &fake_memory[0..=2]).unwrap_err();
        insert_rw_slice(&mut db, &fake_memory[5..=10]).unwrap_err();
        insert_ro_slice(&mut db, &fake_memory[1..=7]).unwrap_err();
        insert_ro_slice(&mut db, &fake_memory[3..=4]).unwrap_err();
        insert_ro_slice(&mut db, &fake_memory[2..=5]).unwrap_err();
        // Add a second buffer, and make sure we can still add a buffer in the
        // middle.
        let mut rw_buffer_8_12 = insert_rw_slice(&mut db, &fake_memory[8..=12]).unwrap();
        let ro_buffer_6_7 = insert_ro_slice(&mut db, &fake_memory[6..=7]).unwrap();

        // Check the Deref implementations on the read-only buffers. For the
        // nonempty read-write buffers, we mutate the buffers as well.
        assert_eq!(*ro_buffer_2_5, [2, 3, 4, 5]);
        assert_eq!(*ro_buffer_3_empty, []);
        assert_eq!(*rw_buffer_3_empty, []);
        assert_eq!(*rw_buffer_8_12, [8, 9, 10, 11, 12]);
        rw_buffer_8_12.copy_from_slice(&[20, 21, 22, 23, 24]);
        assert_eq!(*ro_buffer_6_7, [6, 7]);

        // Remove a buffer, and check it can be re-added.
        remove_ro_check(&mut db, ro_buffer_6_7, &fake_memory[6..=7]);
        let rw_buffer_6_7 = insert_rw_slice(&mut db, &fake_memory[6..=7]).unwrap();

        // Clean up all the buffers.
        remove_ro_check(&mut db, ro_buffer_2_5, &fake_memory[2..=5]);
        remove_ro_check(&mut db, ro_buffer_3_empty, &fake_memory[3..3]);
        remove_rw_check(&mut db, rw_buffer_3_empty, &fake_memory[3..3]);
        remove_rw_check(&mut db, rw_buffer_8_12, &fake_memory[8..=12]);
        remove_rw_check(&mut db, rw_buffer_6_7, &fake_memory[6..=7]);
    }

    // Verify the values were correctly written into fake_memory.
    let expected: &mut [u8] = &mut [0, 1, 2, 3, 4, 5, 6, 7, 20, 21, 22, 23, 24, 13, 14, 15];
    assert_eq!(fake_memory, Cell::from_mut(expected).as_slice_of_cells());
}
