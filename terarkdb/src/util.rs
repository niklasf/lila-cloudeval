use std::{
    ffi::{c_char, c_void, CStr},
    marker::PhantomData,
    ptr::NonNull,
    slice,
};

use terarkdb_sys::rocksdb_free;

#[repr(transparent)]
pub struct Malloced<T> {
    inner: NonNull<T>,
    marker: PhantomData<T>,
}

impl<T> Malloced<T> {
    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_ptr()
    }
}

impl Malloced<c_char> {
    pub unsafe fn as_bytes_with_len(&self, len: usize) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr().cast::<u8>(), len) }
    }

    pub unsafe fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_ptr()) }
    }
}

unsafe impl<T: Send> Send for Malloced<T> {}
unsafe impl<T: Sync> Sync for Malloced<T> {}

impl<T> Drop for Malloced<T> {
    fn drop(&mut self) {
        unsafe {
            rocksdb_free(self.as_mut_ptr().cast::<c_void>());
        }
    }
}
