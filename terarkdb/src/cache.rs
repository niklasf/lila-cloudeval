use std::ptr::NonNull;

use terarkdb_sys::{rocksdb_cache_create_lru, rocksdb_cache_destroy, rocksdb_cache_t};

pub struct Cache {
    inner: NonNull<rocksdb_cache_t>,
}

impl Cache {
    pub fn new_lru(capacity_bytes: usize) -> Cache {
        Cache {
            inner: NonNull::new(unsafe { rocksdb_cache_create_lru(capacity_bytes) }).unwrap(),
        }
    }

    pub(crate) fn as_implied_const_ptr(&self) -> *mut rocksdb_cache_t {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_cache_t {
        self.inner.as_ptr()
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        unsafe {
            // Decrement reference counter of the underlying object.
            rocksdb_cache_destroy(self.as_mut_ptr());
        }
    }
}
