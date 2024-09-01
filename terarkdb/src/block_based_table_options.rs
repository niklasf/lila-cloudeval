use std::ptr::NonNull;

use terarkdb_sys::{
    rocksdb_block_based_options_create, rocksdb_block_based_options_destroy,
    rocksdb_block_based_options_set_block_cache, rocksdb_block_based_table_options_t,
};

use crate::Cache;

pub struct BlockBasedTableOptions {
    inner: NonNull<rocksdb_block_based_table_options_t>,
}

impl Default for BlockBasedTableOptions {
    fn default() -> BlockBasedTableOptions {
        BlockBasedTableOptions::new()
    }
}

impl BlockBasedTableOptions {
    pub fn new() -> BlockBasedTableOptions {
        BlockBasedTableOptions {
            inner: NonNull::new(unsafe { rocksdb_block_based_options_create() }).unwrap(),
        }
    }

    pub fn set_block_cache(&mut self, cache: &Cache) -> &mut Self {
        unsafe {
            rocksdb_block_based_options_set_block_cache(
                self.as_mut_ptr(),
                cache.as_implied_const_ptr(),
            );
        }
        self
    }

    pub(crate) fn as_implied_const_ptr(&self) -> *mut rocksdb_block_based_table_options_t {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&self) -> *mut rocksdb_block_based_table_options_t {
        self.inner.as_ptr()
    }
}

impl Drop for BlockBasedTableOptions {
    fn drop(&mut self) {
        unsafe {
            rocksdb_block_based_options_destroy(self.as_mut_ptr());
        }
    }
}

unsafe impl Send for BlockBasedTableOptions {}
unsafe impl Sync for BlockBasedTableOptions {}
