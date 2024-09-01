use std::mem;

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, CastlingMode, Chess, Color, File, Piece, Rank, Role, Setup, Square};

use cdbdirect::cdb_fen::cdb_fen;

fn bw(mut setup: Setup) -> Setup {
    setup.mirror();
    setup
}

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

        assert_eq!(
            hex::encode(cdb_fen(record.fen.as_setup()).as_bytes()),
            record.cdb_fen,
            "line {}: cdb_fen mismatch for {}",
            i + 2,
            record.fen
        );

        assert_eq!(
            hex::encode(cdb_fen(&bw(record.fen.as_setup().clone())).as_bytes()),
            record.cdb_fen_bw,
            "line {}: cdb_fen_bw mismatch for {}",
            i + 2,
            record.fen
        );
    }
}
