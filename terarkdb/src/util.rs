use std::{ffi::c_void, marker::PhantomData, ptr::NonNull};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_char;
    use std::mem;

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
