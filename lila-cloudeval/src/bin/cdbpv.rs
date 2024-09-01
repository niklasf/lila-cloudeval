use clap::Parser as _;
use lila_cloudeval::database::Database;
use lila_cloudeval::database::DatabaseOpt;
use shakmaty::Chess;
use std::error::Error;

#[derive(Debug, clap::Parser)]
struct Opt {
    #[clap(flatten)]
    db: DatabaseOpt,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::parse();

    let database = Database::open_read_only_blocking(&opt.db)?;

    let pv = database.get_pv_blocking(Chess::default(), usize::MAX)?;

    for uci in pv {
        println!("{uci}");
    }

    Ok(())
}
