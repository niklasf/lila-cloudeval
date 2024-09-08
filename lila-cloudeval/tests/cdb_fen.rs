use std::{cmp::min, fs::File};

use lila_cloudeval::cdb_fen::cdb_fen;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, CastlingMode, Chess};

#[serde_as]
#[derive(Deserialize)]
struct Record {
    #[serde_as(as = "DisplayFromStr")]
    fen: Fen,
    hex_fen: String,
    hex_fen_bw: String,
}

#[test]
fn test_cdb_fen() {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(
            ruzstd::StreamingDecoder::new(File::open("tests/reference.csv.zst").expect("csv zst"))
                .expect("zst"),
        );

    for (i, record) in reader.deserialize().enumerate() {
        let line = i + 1;
        let record: Record = record.expect("record");

        if record
            .fen
            .clone()
            .into_position::<Chess>(CastlingMode::Chess960)
            .is_err()
        {
            continue;
        }

        let (bin_fen, _) = cdb_fen(record.fen.as_setup());

        assert_eq!(
            hex::encode(&bin_fen.as_bytes()[1..]),
            min(record.hex_fen, record.hex_fen_bw),
            "line {}: cdb_fen mismatch for {}",
            line,
            record.fen
        );
    }
}
