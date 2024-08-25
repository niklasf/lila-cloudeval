use terarkdb_sys::rocksdb_options_t;
use terarkdb_sys::rocksdb_options_create;
use terarkdb_sys::rocksdb_options_destroy;
use std::ptr::NonNull;

pub struct Options {
    inner: NonNull<rocksdb_options_t>,
}

impl Options {
    pub fn create() -> Options {
        Options {
            inner: NonNull::new(unsafe {rocksdb_options_create() }).unwrap(),
        }
    }
}

impl Drop for Options {
    fn drop(&mut self) {
        unsafe { rocksdb_options_destroy(self.inner.as_ptr()); }
    }
}
