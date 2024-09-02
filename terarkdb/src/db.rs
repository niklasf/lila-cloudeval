use std::{
    ffi::{c_char, c_uchar, CString},
    path::Path,
    ptr::NonNull,
};

use terarkdb_sys::{
    rocksdb_close, rocksdb_get, rocksdb_get_pinned, rocksdb_multi_get, rocksdb_open,
    rocksdb_open_for_read_only, rocksdb_t,
};

use crate::{
    error::Error, multi_get::MultiGet, options::Options, pinnable_slice::PinnableSlice,
    read_options::ReadOptions, util::Malloced, MallocedBytes,
};

fn cpath(path: &Path) -> CString {
    use std::os::unix::ffi::OsStrExt as _;
    CString::new(path.as_os_str().as_bytes()).expect("no NUL in unix path")
}

#[derive(Default)]
pub enum LogFile {
    #[default]
    Ignore = 0,
    ErrorIfExists = 1,
}

#[derive(Debug)]
pub struct Db {
    inner: NonNull<rocksdb_t>,
}

impl Db {
    pub fn open<P: AsRef<Path>>(options: &Options, path: P) -> Result<Db, Error> {
        let mut error = None;
        let maybe_db = unsafe {
            rocksdb_open(
                options.as_ptr(),
                cpath(path.as_ref()).as_ptr(),
                Error::out_ptr(&mut error),
            )
        };

        error.map_or_else(
            || {
                Ok(Db {
                    inner: NonNull::new(maybe_db).unwrap(),
                })
            },
            Err,
        )
    }

    pub fn open_read_only<P: AsRef<Path>>(
        options: &Options,
        path: P,
        log_file: LogFile,
    ) -> Result<Db, Error> {
        let mut error = None;
        let maybe_db = unsafe {
            rocksdb_open_for_read_only(
                options.as_ptr(),
                cpath(path.as_ref()).as_ptr(),
                log_file as c_uchar,
                Error::out_ptr(&mut error),
            )
        };

        error.map_or_else(
            || {
                Ok(Db {
                    inner: NonNull::new(maybe_db).unwrap(),
                })
            },
            Err,
        )
    }

    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<MallocedBytes>, Error> {
        self.get_opt(key, &ReadOptions::default())
    }

    pub fn get_opt<K: AsRef<[u8]>>(
        &self,
        key: K,
        read_options: &ReadOptions,
    ) -> Result<Option<MallocedBytes>, Error> {
        let key = key.as_ref();
        let mut error = None;
        let mut len = 0;
        let maybe_bytes = unsafe {
            Malloced::new(rocksdb_get(
                self.as_mut_ptr(),
                read_options.as_ptr(),
                key.as_ptr().cast::<c_char>(),
                key.len(),
                &mut len,
                Error::out_ptr(&mut error),
            ))
        };

        error.map_or_else(
            || Ok(maybe_bytes.map(|bytes| unsafe { MallocedBytes::from_parts(bytes, len) })),
            Err,
        )
    }

    pub fn get_pinned<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<PinnableSlice<'_>>, Error> {
        self.get_pinned_opt(key, &ReadOptions::default())
    }

    pub fn get_pinned_opt<K: AsRef<[u8]>>(
        &self,
        key: K,
        read_options: &ReadOptions,
    ) -> Result<Option<PinnableSlice<'_>>, Error> {
        let key = key.as_ref();
        let mut error = None;
        let maybe_slice = unsafe {
            PinnableSlice::new(rocksdb_get_pinned(
                self.as_mut_ptr(),
                read_options.as_ptr(),
                key.as_ptr().cast::<c_char>(),
                key.len(),
                Error::out_ptr(&mut error),
            ))
        };

        error.map_or(Ok(maybe_slice), Err)
    }

    pub fn multi_get<K: AsRef<[u8]>>(&self, keys: &[K]) -> MultiGet {
        self.multi_get_opt(keys, &ReadOptions::default())
    }

    pub fn multi_get_opt<K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        read_options: &ReadOptions,
    ) -> MultiGet {
        let (key_ptrs, key_lens): (Vec<*const c_char>, Vec<usize>) = keys
            .iter()
            .map(|k| {
                let key = k.as_ref();
                (key.as_ptr().cast::<c_char>(), key.len())
            })
            .unzip();

        let mut multi_get = MultiGet::new(keys.len());
        unsafe {
            rocksdb_multi_get(
                self.as_mut_ptr(),
                read_options.as_ptr(),
                keys.len(),
                key_ptrs.as_ptr(),
                key_lens.as_ptr(),
                Malloced::out_ptr(multi_get.values.as_mut_ptr()),
                multi_get.lens.as_mut_ptr(),
                Error::out_ptr(multi_get.errors.as_mut_ptr()),
            );
        }

        multi_get
    }

    pub(crate) fn as_mut_ptr(&self) -> *mut rocksdb_t {
        self.inner.as_ptr()
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        unsafe {
            rocksdb_close(self.as_mut_ptr());
        }
    }
}

unsafe impl Send for Db {}
unsafe impl Sync for Db {}
