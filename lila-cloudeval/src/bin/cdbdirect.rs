#![forbid(unsafe_code)]

use std::{
    error::Error,
    fs::File,
    hint::black_box,
    io::{BufRead as _, BufReader},
    num::NonZeroUsize,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::Instant,
};

use clap::Parser as _;
use lila_cloudeval::{
    cdb_moves::SortedScoredMoves,
    database::{Database, DatabaseOpt},
};
use shakmaty::fen::Fen;

#[derive(Debug, clap::Parser)]
struct Opt {
    #[clap(flatten)]
    db: DatabaseOpt,
    #[clap(long)]
    threads: Option<NonZeroUsize>,
    fens: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::parse();

    let database = Database::open_read_only_blocking(&opt.db)?;

    let threads = opt
        .threads
        .unwrap_or_else(|| thread::available_parallelism().unwrap());

    let found = AtomicU64::new(0);
    let not_found = Mutex::new(Vec::new());
    let total_moves = AtomicU64::new(0);
    let found_ply_from_root = AtomicU64::new(0);

    let started_at = Instant::now();

    rayon::scope(|s| {
        let (tx, rx) = crossbeam_channel::bounded::<String>(1_000_000);

        for _ in 0..usize::from(threads) {
            let database = &database;
            let found = &found;
            let not_found = &not_found;
            let total_moves = &total_moves;
            let found_ply_from_root = &found_ply_from_root;
            let rx = rx.clone();

            s.spawn(move |_| {
                while let Ok(line) = rx.recv() {
                    let setup = line.parse::<Fen>().unwrap().into_setup();

                    if let Some(SortedScoredMoves(scored_moves)) =
                        database.get_blocking(setup).unwrap()
                    {
                        found.fetch_add(1, Ordering::Relaxed);

                        total_moves.fetch_add(scored_moves.len() as u64, Ordering::Relaxed);
                        if scored_moves.ply_from_root().is_some() {
                            found_ply_from_root.fetch_add(1, Ordering::Relaxed);
                        }

                        black_box(&scored_moves);
                        // println!("{line}: {scored_moves:?}");
                    } else {
                        not_found.lock().unwrap().push(line);
                    }
                }
            });
        }

        for path in opt.fens {
            for line in BufReader::new(File::open(path).unwrap()).lines() {
                tx.send(line.unwrap()).unwrap();
            }
        }
    });

    println!("{:.3?} elapased", started_at.elapsed());
    println!("{} found", found.load(Ordering::Relaxed));
    println!("{} missing", not_found.into_inner().unwrap().len());
    println!("{} scored moves", total_moves.load(Ordering::Relaxed));
    println!(
        "{} found with ply from root",
        found_ply_from_root.load(Ordering::Relaxed)
    );

    Ok(())
}
