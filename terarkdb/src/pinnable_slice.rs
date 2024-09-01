use std::{ffi::c_char, marker::PhantomData, ops::Deref, ptr::NonNull, slice};

use terarkdb_sys::{
    rocksdb_pinnableslice_destroy, rocksdb_pinnableslice_t, rocksdb_pinnableslice_value,
};

use crate::db::Db;

#[derive(Debug)]
pub struct PinnableSlice<'db> {
    inner: NonNull<rocksdb_pinnableslice_t>,
    db: PhantomData<&'db Db>,
}

impl PinnableSlice<'_> {
    pub(crate) unsafe fn new<'db>(
        slice_or_null: *mut rocksdb_pinnableslice_t,
    ) -> Option<PinnableSlice<'db>> {
        Some(PinnableSlice {
            inner: NonNull::new(slice_or_null)?,
            db: PhantomData,
        })
    }

    pub(crate) fn as_inner_ptr(&self) -> *const rocksdb_pinnableslice_t {
        self.inner.as_ptr()
    }

    pub(crate) fn as_inner_mut_ptr(&mut self) -> *mut rocksdb_pinnableslice_t {
        self.inner.as_ptr()
    }
}

impl AsRef<[u8]> for PinnableSlice<'_> {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl Deref for PinnableSlice<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            let mut len = 0;
            let value: *const c_char = rocksdb_pinnableslice_value(self.as_inner_ptr(), &mut len);
            slice::from_raw_parts(value.cast::<u8>(), len)
        }
    }
}

impl Drop for PinnableSlice<'_> {
    fn drop(&mut self) {
        unsafe {
            rocksdb_pinnableslice_destroy(self.as_inner_mut_ptr());
        }
    }
}

unsafe impl Send for PinnableSlice<'_> {}
unsafe impl Sync for PinnableSlice<'_> {}
