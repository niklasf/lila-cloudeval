use std::{
    ffi::{c_char, c_uchar, CString},
    path::Path,
    ptr::NonNull,
};

use terarkdb_sys::{
    rocksdb_close, rocksdb_get_pinned, rocksdb_open, rocksdb_open_for_read_only, rocksdb_t,
};

use crate::{
    error::Error, options::Options, pinnable_slice::PinnableSlice, read_options::ReadOptions,
};

fn cpath(path: &Path) -> CString {
    use std::os::unix::ffi::OsStrExt as _;
    CString::new(path.as_os_str().as_bytes()).expect("no NUL in unix path")
}

#[derive(Default)]
pub enum LogFile {
    #[default]
    Ignore = 0,
    ErrorIfExists = 1,
}

pub struct Db {
    inner: NonNull<rocksdb_t>,
}

impl Db {
    pub fn open<P: AsRef<Path>>(options: &Options, path: P) -> Result<Db, Error> {
        let mut error = Error::new();
        let maybe_db = unsafe {
            rocksdb_open(
                options.as_ptr(),
                cpath(path.as_ref()).as_ptr(),
                error.as_mut_ptr(),
            )
        };

        error.or_else(|| Db {
            inner: NonNull::new(maybe_db).unwrap(),
        })
    }

    pub fn open_for_readonly<P: AsRef<Path>>(
        options: &Options,
        path: P,
        log_file: LogFile,
    ) -> Result<Db, Error> {
        let mut error = Error::new();
        let maybe_db = unsafe {
            rocksdb_open_for_read_only(
                options.as_ptr(),
                cpath(path.as_ref()).as_ptr(),
                log_file as c_uchar,
                error.as_mut_ptr(),
            )
        };

        error.or_else(|| Db {
            inner: NonNull::new(maybe_db).unwrap(),
        })
    }

    pub fn get<'db, K: AsRef<[u8]>>(
        &'db self,
        key: K,
    ) -> Result<Option<PinnableSlice<'db>>, Error> {
        self.get_opt(key, &ReadOptions::default())
    }

    pub fn get_opt<'db, K: AsRef<[u8]>>(
        &'db self,
        key: K,
        read_options: &ReadOptions,
    ) -> Result<Option<PinnableSlice<'db>>, Error> {
        let key = key.as_ref();
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
