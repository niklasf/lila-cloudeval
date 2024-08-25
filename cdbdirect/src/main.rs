#![forbid(unsafe_op_in_unsafe_fn)]

use terarkdb::Db;
use terarkdb::Options;

fn main() {
    let mut options = Options::create();
    options.increase_parallelism();

    let db = Db::new();
}
