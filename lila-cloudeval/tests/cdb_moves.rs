use std::{collections::HashMap, fs::File};

use lila_cloudeval::{
    cdb_fen::NaturalOrder,
    cdb_moves::{RelativeScore, ScoredMoves},
};
use shakmaty::uci::UciMove;

#[test]
fn test_cdb_moves() {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(
            ruzstd::StreamingDecoder::new(File::open("tests/reference.csv.zst").expect("csv zst"))
                .expect("zst"),
        );

    for (i, record) in reader.records().enumerate() {
        let line = i + 1;
        let record = record.expect("record");
        let mut fields = record.into_iter();

        let _fen = fields.next().expect("fen");
        let hex_fen = fields.next().expect("hex fen");
        let hex_fen_bw = fields.next().expect("hex fen bw");
        let natural_order = if hex_fen < hex_fen_bw {
            NaturalOrder::Same
        } else {
            NaturalOrder::Mirror
        };
        let value = hex::decode(fields.next().expect("hex value")).expect("value");
        let ply_from_root = Some(fields.next().expect("ply from root"))
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<u32>().expect("ply from root int"));
        let expected_moves = {
            let mut moves = HashMap::new();
            while let Some(uci) = fields.next() {
                moves.insert(
                    uci.parse::<UciMove>().expect("uci"),
                    RelativeScore(
                        fields
                            .next()
                            .expect("score")
                            .parse::<i16>()
                            .expect("score int"),
                    ),
                );
            }
            moves
        };

        let scored_moves = ScoredMoves::read_cdb(&mut &value[..], natural_order);
        let actual_moves: HashMap<_, _> = scored_moves
            .moves()
            .into_iter()
            .map(|e| (e.uci.clone(), e.score))
            .collect();

        assert_eq!(
            scored_moves.ply_from_root(),
            ply_from_root,
            "line {line}: ply from root mismatch"
        );

        assert_eq!(actual_moves, expected_moves, "line {line}");
    }
}
