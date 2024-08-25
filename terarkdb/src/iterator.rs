use std::{marker::PhantomData, ptr::NonNull};

use terarkdb_sys::{rocksdb_create_iterator, rocksdb_iter_destroy, rocksdb_iterator_t};

use crate::{db::Db, read_options::ReadOptions};

pub struct Iterator<'db> {
    inner: NonNull<rocksdb_iterator_t>,
    db: PhantomData<&'db Db>,
}

impl Iterator<'_> {
    pub fn new<'db>(db: &'db Db, options: &ReadOptions) -> Iterator<'db> {
        Iterator {
            inner: NonNull::new(unsafe {
                rocksdb_create_iterator(db.as_mut_ptr(), options.as_ptr())
            })
            .unwrap(),
            db: PhantomData,
        }
    }

    pub(crate) fn as_ptr(&self) -> *const rocksdb_iterator_t {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_iterator_t {
        self.inner.as_ptr()
    }
}

impl Drop for Iterator<'_> {
    fn drop(&mut self) {
        unsafe { rocksdb_iter_destroy(self.as_mut_ptr()) };
    }
}
