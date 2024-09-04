use std::error::Error;

use clap::Parser as _;
use lila_cloudeval::{
    cdb_moves::ScoredMove,
    database::{Database, DatabaseOpt},
};
use shakmaty::{Chess, Setup};

#[derive(Debug, clap::Parser)]
struct Opt {
    #[clap(flatten)]
    db: DatabaseOpt,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::parse();

    let database = Database::open_read_only_blocking(&opt.db)?;

    let root = database.get_blocking(Setup::default())?.unwrap();

    for ScoredMove { uci, score } in root.moves() {
        println!("{uci}: {score}")
    }

    println!("---");

    let pv = database.get_pv_blocking(Chess::default())?;

    for uci in pv {
        println!("{uci}");
    }

    Ok(())
}
