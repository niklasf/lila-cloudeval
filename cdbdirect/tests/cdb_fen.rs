use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, CastlingMode, Chess};

use cdbdirect::cdb_fen::cdb_fen;

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

        let bin_fen = cdb_fen(record.fen.as_setup());
        let bin_fen_bw = cdb_fen(&record.fen.as_setup().clone().into_mirrored());

        assert_eq!(
            hex::encode(bin_fen.as_bytes()),
            record.cdb_fen,
            "line {}: cdb_fen mismatch for {}",
            i + 2,
            record.fen
        );

        assert_eq!(
            hex::encode(bin_fen_bw.as_bytes()),
            record.cdb_fen_bw,
            "line {}: cdb_fen_bw mismatch for {}",
            i + 2,
            record.fen
        );

        assert_eq!(
            bin_fen.as_bytes() < bin_fen_bw.as_bytes(),
            record.cdb_fen < record.cdb_fen_bw,
            "line {}: natural order mismatch for {}",
            i + 2,
            record.fen
        );
    }
}
