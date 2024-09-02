use core::slice;
use std::{
    ffi::{c_char, c_void},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use terarkdb_sys::rocksdb_free;

/// Non-null `*mut T` representing an owned `T` that can be freed with
/// `rocksdb_free()`.
#[derive(Debug)]
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

    pub fn out_ptr(value: *mut Option<Malloced<T>>) -> *mut *mut T {
        value as _
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

pub struct MallocedBytes {
    ptr: Malloced<c_char>,
    len: usize,
}

impl MallocedBytes {
    pub unsafe fn from_parts(ptr: Malloced<c_char>, len: usize) -> MallocedBytes {
        MallocedBytes { ptr, len }
    }
}

impl AsRef<[u8]> for MallocedBytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl AsMut<[u8]> for MallocedBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self
    }
}

impl Deref for MallocedBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr().cast::<u8>(), self.len) }
    }
}

impl DerefMut for MallocedBytes {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_mut_ptr().cast::<u8>(), self.len) }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::c_char, mem};

    use super::*;

    #[test]
    fn test_malloced_repr() {
        assert_eq!(
            mem::size_of::<Option<Malloced<c_char>>>(),
            mem::size_of::<*mut c_char>()
        );

        assert_eq!(
            mem::align_of::<Option<Malloced<c_char>>>(),
            mem::align_of::<*mut c_char>()
        );
    }
}
