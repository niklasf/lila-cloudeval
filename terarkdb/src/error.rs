use std::{
    error::Error as StdError,
    ffi::{c_char, CStr},
    fmt,
};

use crate::util::Malloced;

#[repr(transparent)]
pub struct Error {
    inner: Malloced<c_char>,
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

impl StdError for Error {}

impl Error {
    pub(crate) fn as_cstr(&self) -> &CStr {
        unsafe { self.inner.as_cstr() }
    }

    pub(crate) fn out_ptr(error: &mut Option<Error>) -> *mut *mut c_char {
        let p: *mut Option<Error> = error;
        p as _
    }
}
