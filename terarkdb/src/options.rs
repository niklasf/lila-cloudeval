use std::{ffi::c_int, num::NonZeroUsize, ptr::NonNull};

use terarkdb_sys::{
    rocksdb_options_create, rocksdb_options_destroy, rocksdb_options_increase_parallelism,
    rocksdb_options_t,
};

pub struct Options {
    inner: NonNull<rocksdb_options_t>,
}

impl Options {
    pub fn create() -> Options {
        Options {
            inner: NonNull::new(unsafe { rocksdb_options_create() }).unwrap(),
        }
    }

    pub fn increase_parallelism(&mut self, total_threads: NonZeroUsize) {
        unsafe {
            rocksdb_options_increase_parallelism(
                self.inner.as_ptr(),
                c_int::try_from(usize::from(total_threads)).unwrap(),
            );
        };
    }
}

impl Drop for Options {
    fn drop(&mut self) {
        unsafe {
            rocksdb_options_destroy(self.inner.as_ptr());
        }
    }
}
