use libtock_platform::{CommandReturn, ErrorCode};

use core::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::str;

use crate::{DriverInfo, DriverShareRef, RoAllowBuffer, RwAllowBuffer};

pub struct KeyValue {
    buffer_in_key: Cell<RoAllowBuffer>,
    buffer_in_val: Cell<RoAllowBuffer>,

    buffer_out_val: RefCell<RwAllowBuffer>,

    share_ref: DriverShareRef,

    database: RefCell<HashMap<String, String>>,
}

impl KeyValue {
    pub fn new() -> std::rc::Rc<KeyValue> {
        std::rc::Rc::new(KeyValue {
            buffer_in_key: Default::default(),
            buffer_in_val: Default::default(),
            buffer_out_val: Default::default(),

            share_ref: Default::default(),

            database: Default::default(),
        })
    }
}

impl crate::fake::SyscallDriver for KeyValue {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(1)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn allow_readonly(
        &self,
        buffer_num: u32,
        buffer: RoAllowBuffer,
    ) -> Result<RoAllowBuffer, (RoAllowBuffer, ErrorCode)> {
        match buffer_num {
            RO_ALLOW_KEY => Ok(self.buffer_in_key.replace(buffer)),
            RO_ALLOW_VAL => Ok(self.buffer_in_val.replace(buffer)),
            _ => Err((buffer, ErrorCode::Invalid)),
        }
    }

    fn allow_readwrite(
        &self,
        buffer_num: u32,
        buffer: RwAllowBuffer,
    ) -> Result<RwAllowBuffer, (RwAllowBuffer, ErrorCode)> {
        match buffer_num {
            RW_ALLOW_VAL => Ok(self.buffer_out_val.replace(buffer)),
            _ => Err((buffer, ErrorCode::Invalid)),
        }
    }

    fn command(&self, command_id: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_id {
            CMD_DRIVER_CHECK => crate::command_return::success(),
            CMD_GET => {
                let k = self.buffer_in_key.take();
                let k_str = str::from_utf8(&k).unwrap();

                let db = self.database.take();
                match db.get(k_str) {
                    Some(val) => {
                        let cp_len = core::cmp::min(self.buffer_out_val.borrow().len(), val.len());
                        self.buffer_out_val.borrow_mut()[..cp_len]
                            .copy_from_slice(&val.as_bytes()[..cp_len]);

                        self.share_ref
                            .schedule_upcall(SUB_CALLBACK, (0, val.len() as u32, 0))
                            .expect("Unable to schedule upcall {}");
                    }
                    _ => {
                        self.share_ref
                            .schedule_upcall(SUB_CALLBACK, (ErrorCode::NoSupport as u32, 0, 0))
                            .expect("Unable to schedule upcall {}");
                    }
                }
                self.buffer_in_key.set(k);
                self.database.replace(db);

                crate::command_return::success()
            }
            CMD_SET => {
                let k = self.buffer_in_key.take();
                let k_str = str::from_utf8(&k).unwrap();
                let v = self.buffer_in_val.take();
                let v_str = str::from_utf8(&v).unwrap();

                let mut db = self.database.take();
                db.insert(k_str.to_string(), v_str.to_string());

                self.buffer_in_key.set(k);
                self.buffer_in_val.set(v);
                self.database.replace(db);

                self.share_ref
                    .schedule_upcall(SUB_CALLBACK, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");

                crate::command_return::success()
            }
            CMD_ADD => {
                let k = self.buffer_in_key.take();
                let k_str = str::from_utf8(&k).unwrap();
                let v = self.buffer_in_val.take();
                let v_str = str::from_utf8(&v).unwrap();

                let mut db = self.database.take();

                let mut found = false;
                if let Some(_val) = db.get(k_str) {
                    self.share_ref
                        .schedule_upcall(SUB_CALLBACK, (ErrorCode::NoSupport as u32, 0, 0))
                        .expect("Unable to schedule upcall {}");
                    found = true;
                }

                if !found {
                    db.insert(k_str.to_string(), v_str.to_string());

                    self.share_ref
                        .schedule_upcall(SUB_CALLBACK, (0, 0, 0))
                        .expect("Unable to schedule upcall {}");
                }

                self.buffer_in_key.set(k);
                self.buffer_in_val.set(v);
                self.database.replace(db);

                crate::command_return::success()
            }
            CMD_UPDATE => {
                let k = self.buffer_in_key.take();
                let k_str = str::from_utf8(&k).unwrap();
                let v = self.buffer_in_val.take();
                let v_str = str::from_utf8(&v).unwrap();

                let mut db = self.database.take();

                let mut found = false;
                match db.get(k_str) {
                    Some(_val) => {
                        found = true;
                    }
                    _ => {
                        self.share_ref
                            .schedule_upcall(SUB_CALLBACK, (ErrorCode::NoSupport as u32, 0, 0))
                            .expect("Unable to schedule upcall {}");
                    }
                }

                if found {
                    db.insert(k_str.to_string(), v_str.to_string());

                    self.share_ref
                        .schedule_upcall(SUB_CALLBACK, (0, 0, 0))
                        .expect("Unable to schedule upcall {}");
                }

                self.buffer_in_key.set(k);
                self.buffer_in_val.set(v);
                self.database.replace(db);

                crate::command_return::success()
            }
            CMD_DELETE => {
                let k = self.buffer_in_key.take();
                let k_str = str::from_utf8(&k).unwrap();

                let mut db = self.database.take();

                match db.remove(k_str) {
                    Some(_val) => {
                        self.share_ref
                            .schedule_upcall(SUB_CALLBACK, (0, 0, 0))
                            .expect("Unable to schedule upcall {}");
                    }
                    _ => {
                        self.share_ref
                            .schedule_upcall(SUB_CALLBACK, (ErrorCode::NoSupport as u32, 0, 0))
                            .expect("Unable to schedule upcall {}");
                    }
                }

                self.buffer_in_key.set(k);
                self.database.replace(db);

                crate::command_return::success()
            }
            _ => crate::command_return::failure(ErrorCode::NoSupport),
        }
    }
}

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x50003;

// Command IDs

const CMD_DRIVER_CHECK: u32 = 0;
const CMD_GET: u32 = 1;
const CMD_SET: u32 = 2;
const CMD_DELETE: u32 = 3;
const CMD_ADD: u32 = 4;
const CMD_UPDATE: u32 = 5;

const RO_ALLOW_KEY: u32 = 0;
const RO_ALLOW_VAL: u32 = 1;
const RW_ALLOW_VAL: u32 = 0;

const SUB_CALLBACK: u32 = 0;
