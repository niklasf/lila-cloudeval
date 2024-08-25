#![forbid(unsafe_op_in_unsafe_fn)]

use terarkdb::{Db, Iterator, Options, ReadOptions};

fn main() {
    let mut options = Options::new();
    options.increase_parallelism(16);

    let db = Db::open_for_readonly(&options, c"/mnt/ssd/chess-20240814/data", false).unwrap();

    let read_options = ReadOptions::default();
    let mut iterator = Iterator::new(&db, &read_options);
    iterator.seek_to_first();
    while let Some((key, value)) = iterator.item() {
        println!("{key:?}: {value:?}");
        iterator.next();
    }

    iterator.status().unwrap();
}
