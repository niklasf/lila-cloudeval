#![forbid(unsafe_op_in_unsafe_fn)]

use cdbdirect::cdb_fen::cdb_fen;
use shakmaty::fen::Fen;
use shakmaty::Setup;
use std::error::Error;
use std::fs::File;
use std::io::BufRead as _;
use std::io::BufReader;
use terarkdb::{Db, Iterator, Options, ReadOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = Options::new();
    options.increase_parallelism(16);

    let db = Db::open_for_readonly(&options, c"/mnt/ssd/chess-20240814/data", false).unwrap();

    let read_options = ReadOptions::default();

    for line in BufReader::new(File::open("/root/lila-cloudeval-bench/fens.txt")?).lines() {
        let setup = line?.parse::<Fen>()?.into_setup();
    }

    let mut iterator = Iterator::new(&db, &read_options);
    iterator.seek_to_first();
    while let Some((key, value)) = iterator.item() {
        println!("{key:?}: {value:?}");
        iterator.next();
    }

    iterator.status().unwrap();

    Ok(())
}
