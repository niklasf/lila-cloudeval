use std::{
    error::Error,
    fs::File,
    io,
    io::{BufRead as _, BufReader},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use rayon::prelude::*;
use reqwest::blocking::Client;

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

    // With RAYON_NUM_THREADS
    fens.par_iter().for_each(|fen| {
        let before = Instant::now();
        let res = client
            .get(&opt.endpoint)
            .query(&[("fen", fen)])
            .send()
            .expect("send");

        if res.status()
            .error_for_status()
            .expect("ok")
            .text()
            .expect("text");
        println!("{}\t{}", fen, before.elapsed().as_millis());
    });

    Ok(())
}
