use std::{
    ffi::{c_char, c_uchar, CStr},
    ptr::NonNull,
};

use terarkdb_sys::{rocksdb_close, rocksdb_get_pinned, rocksdb_open_for_read_only, rocksdb_t};

use crate::{
    error::Error, options::Options, pinnable_slice::PinnableSlice, read_options::ReadOptions,
};

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
        let maybe_db = unsafe {
            rocksdb_open_for_read_only(
                options.as_ptr(),
                name.as_ptr(),
                c_uchar::from(error_if_log_file_exists),
                error.as_mut_ptr(),
            )
        };

        error.or_else(|| Db {
            inner: NonNull::new(maybe_db).unwrap(),
        })
    }

    pub fn get<'db>(
        &'db self,
        key: &[u8],
        read_options: &ReadOptions,
    ) -> Result<Option<PinnableSlice<'db>>, Error> {
        let mut error = Error::new();
        let maybe_slice = unsafe {
            PinnableSlice::new(rocksdb_get_pinned(
                self.as_mut_ptr(),
                read_options.as_ptr(),
                key.as_ptr().cast::<c_char>(),
                key.len(),
                error.as_mut_ptr(),
            ))
        };

        error.or(maybe_slice)
    }

    pub(crate) fn as_mut_ptr(&self) -> *mut rocksdb_t {
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
