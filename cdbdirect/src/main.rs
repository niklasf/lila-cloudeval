#![forbid(unsafe_op_in_unsafe_fn)]

use std::{
    error::Error,
    fs::File,
    io::{BufRead as _, BufReader},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use cdbdirect::{
    cdb_fen::{push_cdb_fen, Nibbles},
    cdb_moves::read_cdb_moves,
};
use shakmaty::fen::Fen;
use terarkdb::{BlockBasedTableOptions, Cache, Db, LogFile, Options, ReadOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let db = Db::open_for_readonly(
        Options::default()
            .increase_parallelism(16)
            .set_block_based_table_options(
                &BlockBasedTableOptions::default()
                    .set_block_cache(&Cache::new_lru(100 * 1024 * 1024)),
            ),
        "/mnt/ssd/chess-20240814/data",
        LogFile::Ignore,
    )?;

    let started_at = Instant::now();

    let found = AtomicU64::new(0);
    let not_found = AtomicU64::new(0);
    let total_moves = AtomicU64::new(0);

    rayon::scope(|s| {
        let (tx, rx) = crossbeam_channel::bounded::<String>(10_000);

        for _ in 0..48 {
            let db = &db;
            let found = &found;
            let not_found = &not_found;
            let total_moves = &total_moves;
            let rx = rx.clone();

            s.spawn(move |_| {
                let read_options = ReadOptions::default();

                let mut bin_fen = Nibbles::new();
                let mut bin_fen_bw = Nibbles::new();

                while let Ok(line) = rx.recv() {
                    let mut setup = line.parse::<Fen>().unwrap().into_setup();

                    bin_fen.clear();
                    push_cdb_fen(&mut bin_fen, &setup);

                    setup.mirror();
                    bin_fen_bw.clear();
                    push_cdb_fen(&mut bin_fen_bw, &setup);

                    let natural_order = bin_fen.as_bytes() < bin_fen_bw.as_bytes();

                    let value = db
                        .get_opt(
                            if natural_order {
                                bin_fen.as_bytes()
                            } else {
                                bin_fen_bw.as_bytes()
                            },
                            &read_options,
                        )
                        .unwrap();

                    if let Some(value) = value {
                        let (scored_moves, ply) = read_cdb_moves(&mut &value[..]);
                        found.fetch_add(1, Ordering::Relaxed);
                        total_moves.fetch_add(scored_moves.len() as u64, Ordering::Relaxed);
                    } else {
                        not_found.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }

        for line in
            BufReader::new(File::open("/root/lila-cloudeval-bench/fens.txt").unwrap()).lines()
        {
            tx.send(line.unwrap()).unwrap();
        }
    });

    println!("{:.3?} elpased", started_at.elapsed());
    println!("{} found", found.load(Ordering::Relaxed));
    println!("{} missing", not_found.load(Ordering::Relaxed));
    println!("{} scored moves", total_moves.load(Ordering::Relaxed));

    Ok(())
}
