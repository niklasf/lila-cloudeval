#![forbid(unsafe_op_in_unsafe_fn)]

use cdbdirect::cdb_fen::push_cdb_fen;
use cdbdirect::cdb_fen::Nibbles;
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

    let mut bin_fen = Nibbles::new();
    let mut bin_fen_bw = Nibbles::new();

    let mut reader = BufReader::new(File::open("/root/lila-cloudeval-bench/fens.txt")?);
    let mut line = Vec::new();
    while reader.read_until(b'\n', &mut line)? != 0 {
        if line[line.len() - 1] == b'\n' {
            line.pop();
        }

        let mut setup = Fen::from_ascii(&line)?.into_setup();

        bin_fen.clear();
        push_cdb_fen(&mut bin_fen, &setup);

        setup.mirror();
        bin_fen_bw.clear();
        push_cdb_fen(&mut bin_fen_bw, &setup);

        let natural_order = bin_fen.as_bytes() < bin_fen_bw.as_bytes();

        let value = db.get_opt(
            if natural_order {
                bin_fen.as_bytes()
            } else {
                bin_fen_bw.as_bytes()
            },
            &read_options,
        )?;

        match value {
            Some(_) => found += 1,
            None => not_found += 1,
        }

        line.clear();
    }

    println!("{found} found");
    println!("{not_found} missing");

    Ok(())
}
