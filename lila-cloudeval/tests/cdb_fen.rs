use std::cmp::min;

use lila_cloudeval::cdb_fen::cdb_fen;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, CastlingMode, Chess};

#[serde_as]
#[derive(Deserialize)]
struct Record {
    #[serde_as(as = "DisplayFromStr")]
    fen: Fen,
    cdb_fen: String,
    cdb_fen_bw: String,
}

#[test]
fn test_cdb_fen() {
    let mut reader = csv::Reader::from_path("tests/cdb_fen_all.csv").expect("reader");
    for (i, record) in reader.deserialize().enumerate() {
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
            min(record.cdb_fen, record.cdb_fen_bw),
            "line {}: cdb_fen mismatch for {}",
            i + 2,
            record.fen
        );
    }
}
