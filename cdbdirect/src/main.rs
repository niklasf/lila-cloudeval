#![forbid(unsafe_op_in_unsafe_fn)]

use cdbdirect::cdb_fen::cdb_fen;
use shakmaty::fen::Fen;
use std::error::Error;
use std::fs::File;
use std::io::BufRead as _;
use std::io::BufReader;
use terarkdb::{Db, Options, ReadOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = Options::new();
    options.increase_parallelism(16);

    let db = Db::open_for_readonly(&options, c"/mnt/ssd/chess-20240814/data", false).unwrap();

    let mut found = 0;
    let mut not_found = 0;

    let read_options = ReadOptions::default();

    for line in BufReader::new(File::open("/root/lila-cloudeval-bench/fens.txt")?).lines() {
        let mut setup = line?.parse::<Fen>()?.into_setup();
        let bin_fen = cdb_fen(&setup);
        setup.mirror();
        let bin_fen_bw = cdb_fen(&setup);
        let natural_order = bin_fen.as_bytes() < bin_fen_bw.as_bytes();

        let value = db.get(if natural_order {
            bin_fen.as_bytes()
        } else {
            bin_fen_bw.as_bytes()
        })?;

        match value {
            Some(_) => found += 1,
            None => not_found += 1,
        }
    }

    println!("{found} found");
    println!("{not_found} missing");

    Ok(())
}
