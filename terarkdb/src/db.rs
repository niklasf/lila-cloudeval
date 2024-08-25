use std::{
    ffi::{c_uchar, CStr},
    ptr::NonNull,
};

use terarkdb_sys::{rocksdb_open_for_read_only, rocksdb_t};

use crate::{error::Error, options::Options};

pub struct Db {
    inner: NonNull<rocksdb_t>,
}

impl Db {
    pub fn open_for_readonly(
        options: &Options,
        name: &CStr,
        error_if_log_file_exists: bool,
    ) -> Result<Db, Error> {
        let mut error = Error::new();
        Ok(Db {
            inner: NonNull::new(unsafe {
                rocksdb_open_for_read_only(
                    options.as_ptr(),
                    name.as_ptr(),
                    c_uchar::from(error_if_log_file_exists),
                    error.as_mut_ptr(),
                )
            })
            .ok_or(error)?,
        })
    }
}
