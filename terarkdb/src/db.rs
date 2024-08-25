use std::{
    ffi::{c_uchar, CStr},
    ptr::NonNull,
};

use terarkdb_sys::{rocksdb_close, rocksdb_open_for_read_only, rocksdb_t};

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

    pub fn as_ptr(&self) -> *const rocksdb_t {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&self) -> *mut rocksdb_t {
        self.inner.as_ptr()
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        unsafe {
            rocksdb_close(self.as_mut_ptr());
        }
    }
}
