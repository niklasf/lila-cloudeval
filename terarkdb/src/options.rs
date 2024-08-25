use std::{ffi::c_int, ptr::NonNull};

use terarkdb_sys::{
    rocksdb_options_create, rocksdb_options_destroy, rocksdb_options_increase_parallelism,
    rocksdb_options_t,
};

pub struct Options {
    inner: NonNull<rocksdb_options_t>,
}

impl Default for Options {
    fn default() -> Options {
        Options::new()
    }
}

impl Options {
    pub fn new() -> Options {
        Options {
            inner: NonNull::new(unsafe { rocksdb_options_create() }).unwrap(),
        }
    }

    pub fn increase_parallelism(&mut self, total_threads: usize) {
        assert!(total_threads >= 1);
        unsafe {
            rocksdb_options_increase_parallelism(
                self.inner.as_ptr(),
                c_int::try_from(total_threads).unwrap(),
            );
        };
    }

    pub fn as_ptr(&self) -> *const rocksdb_options_t {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut rocksdb_options_t {
        self.inner.as_ptr()
    }
}

impl Drop for Options {
    fn drop(&mut self) {
        unsafe {
            rocksdb_options_destroy(self.as_mut_ptr());
        }
    }
}
