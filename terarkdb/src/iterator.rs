use std::{ffi::c_char, marker::PhantomData, ptr::NonNull, slice};

use terarkdb_sys::{
    rocksdb_create_iterator, rocksdb_iter_destroy, rocksdb_iter_get_error, rocksdb_iter_key,
    rocksdb_iter_seek_to_first, rocksdb_iter_seek_to_last, rocksdb_iter_valid, rocksdb_iter_value,
    rocksdb_iterator_t,
};

use crate::{db::Db, error::Error, read_options::ReadOptions};

pub struct Iterator<'db, 'options> {
    inner: NonNull<rocksdb_iterator_t>,
    db: PhantomData<&'db Db>,
    options: PhantomData<&'options ReadOptions>, // for bounds
}

impl<'db, 'options> Iterator<'db, 'options> {
    pub fn new(db: &'db Db, options: &'options ReadOptions) -> Iterator<'db, 'options> {
        Iterator {
            inner: NonNull::new(unsafe {
                rocksdb_create_iterator(db.as_mut_ptr(), options.as_ptr())
            })
            .unwrap(),
            db: PhantomData,
            options: PhantomData,
        }
    }

    pub fn valid(&self) -> bool {
        unsafe { rocksdb_iter_valid(self.as_ptr()) != 0 }
    }

    pub fn status(&self) -> Result<(), Error> {
        let mut error = Error::new();
        unsafe {
            rocksdb_iter_get_error(self.as_ptr(), error.as_mut_ptr());
        }
        if error.is_null() {
            Ok(())
        } else {
            Err(error)
        }
    }

    pub fn seek_to_first(&mut self) {
        unsafe {
            rocksdb_iter_seek_to_first(self.as_mut_ptr());
        }
    }

    pub fn seek_to_last(&mut self) {
        unsafe {
            rocksdb_iter_seek_to_last(self.as_mut_ptr());
        };
    }

    pub unsafe fn key_unchecked(&self) -> &[u8] {
        let mut key_len = 0;
        let key: *const c_char = unsafe { rocksdb_iter_key(self.as_ptr(), &mut key_len) };
        debug_assert!(!key.is_null());
        unsafe { slice::from_raw_parts(key.cast::<u8>(), key_len) }
    }

    pub fn key(&self) -> Option<&[u8]> {
        if self.valid() {
            Some(unsafe { self.key_unchecked() })
        } else {
            None
        }
    }

    pub unsafe fn value_unchecked(&self) -> &[u8] {
        let mut value_len = 0;
        let value: *const c_char = unsafe { rocksdb_iter_value(self.as_ptr(), &mut value_len) };
        debug_assert!(!value.is_null());
        unsafe { slice::from_raw_parts(value.cast::<u8>(), value_len) }
    }

    pub fn value(&self) -> Option<&[u8]> {
        if self.valid() {
            Some(unsafe { self.value_unchecked() })
        } else {
            None
        }
    }

    pub fn item(&self) -> Option<(&[u8], &[u8])> {
        if self.valid() {
            unsafe { Some((self.key_unchecked(), self.value_unchecked())) }
        } else {
            None
        }
    }

    pub(crate) fn as_ptr(&self) -> *const rocksdb_iterator_t {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_iterator_t {
        self.inner.as_ptr()
    }
}

impl Drop for Iterator<'_, '_> {
    fn drop(&mut self) {
        unsafe { rocksdb_iter_destroy(self.as_mut_ptr()) };
    }
}
