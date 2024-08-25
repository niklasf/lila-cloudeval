use terarkdb_sys::rocksdb_t;

pub struct Db {
    inner: *mut rocksdb_t,
}

impl Db {
    pub fn open_for_readonly() -> Db {
        todo!()
    }
}
