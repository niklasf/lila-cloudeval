use std::ptr::NonNull;

use terarkdb_sys::{
    rocksdb_readoptions_create, rocksdb_readoptions_destroy, rocksdb_readoptions_t,
};

pub struct ReadOptions {
    inner: NonNull<rocksdb_readoptions_t>,
}

impl Default for ReadOptions {
    fn default() -> ReadOptions {
        ReadOptions::new()
    }
}

impl ReadOptions {
    pub fn new() -> ReadOptions {
        ReadOptions {
            inner: NonNull::new(unsafe { rocksdb_readoptions_create() }).unwrap(),
        }
    }

    pub(crate) fn as_ptr(&self) -> *const rocksdb_readoptions_t {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_readoptions_t {
        self.inner.as_ptr()
    }
}

impl Drop for ReadOptions {
    fn drop(&mut self) {
        unsafe {
            rocksdb_readoptions_destroy(self.as_mut_ptr());
        }
    }
}
