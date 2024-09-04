use std::{
    error::Error,
    fs::File,
    io,
    io::{BufRead as _, BufReader},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use clap::Parser;
use rayon::prelude::*;
use reqwest::{blocking::Client, StatusCode};

#[derive(clap::Parser)]
struct Opt {
    #[clap(long)]
    endpoint: String,
    #[clap(long, default_value = "1")]
    multi_pv: usize,
    fens: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::parse();

    let fens: Vec<String> = BufReader::new(File::open(opt.fens)?)
        .lines()
        .collect::<Result<_, io::Error>>()?;

    let client = Client::builder()
        .timeout(None)
        .pool_idle_timeout(None)
        .build()
        .expect("client");

    let oks = AtomicU64::new(0);
    let bad_requests = AtomicU64::new(0);
    let others = AtomicU64::new(0);

    // With RAYON_NUM_THREADS
    fens.par_iter().for_each(|fen| {
        let before = Instant::now();

        let res = client
            .get(&opt.endpoint)
            .query(&[("fen", fen)])
            .send()
            .expect("send");

        (match res.status() {
            StatusCode::OK => &oks,
            StatusCode::BAD_REQUEST => &bad_requests,
            _ => &others,
        })
        .fetch_add(1, Ordering::Relaxed);

        let _ = res.text().expect("text");
        println!("{}\t{}", fen, before.elapsed().as_millis());
    });

    eprintln!("ok: {}", oks.load(Ordering::Relaxed));
    eprintln!("bad requests: {}", bad_requests.load(Ordering::Relaxed));
    eprintln!("others: {}", others.load(Ordering::Relaxed));

    Ok(())
}
