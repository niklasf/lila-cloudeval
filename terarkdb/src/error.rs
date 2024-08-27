use std::{
    ffi::{c_char, c_void, CStr},
    fmt, ptr,
};

use terarkdb_sys::rocksdb_free;

pub struct Error {
    inner: *mut c_char,
}

impl Default for Error {
    fn default() -> Error {
        Error::new()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_cstr().fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_cstr().to_bytes().escape_ascii().fmt(f)
    }
}

impl Error {
    pub fn new() -> Error {
        Error {
            inner: ptr::null_mut(),
        }
    }

    pub(crate) fn or<T>(self, value: T) -> Result<T, Error> {
        if self.is_null() {
            Ok(value)
        } else {
            Err(self)
        }
    }

    pub(crate) fn or_else<T, F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> T,
    {
        if self.is_null() {
            Ok(f())
        } else {
            Err(self)
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut *mut c_char {
        &mut self.inner
    }

    pub(crate) fn as_cstr(&self) -> &CStr {
        if self.is_null() {
            c"no error"
        } else {
            unsafe { CStr::from_ptr(self.inner) }
        }
    }
}

impl Drop for Error {
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe {
                rocksdb_free(self.inner.cast::<c_void>());
            }
        }
    }
}
