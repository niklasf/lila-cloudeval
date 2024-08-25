#![forbid(unsafe_op_in_unsafe_fn)]

use terarkdb::{Db, Options};

fn main() {
    let mut options = Options::new();
    options.increase_parallelism(16);

    let db = Db::open_for_readonly(&options, c"/mnt/ssd/chess-20240814/data", false).unwrap();
}
