use bytes::Buf;
use shakmaty::uci::UciMove;
use shakmaty::Role;
use shakmaty::Square;
use shakmaty::{File, Rank};

use File::*;
use Rank::*;
use Square::*;

#[rustfmt::skip]
const DEC_FILE: [Option<File>; 90] = [
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
    Some(A), Some(B), Some(C), Some(D), Some(E), Some(F), Some(G), Some(H), None,
];

#[rustfmt::skip]
const DEC_RANK: [Option<Rank>; 90] = [
    None,          None,          None,          None,          None,          None,          None,          None,          None,
    Some(First),   Some(First),   Some(First),   Some(First),   Some(First),   Some(First),   Some(First),   Some(First),   Some(First),
    Some(Second),  Some(Second),  Some(Second),  Some(Second),  Some(Second),  Some(Second),  Some(Second),  Some(Second),  Some(Second),
    Some(Third),   Some(Third),   Some(Third),   Some(Third),   Some(Third),   Some(Third),   Some(Third),   Some(Third),   Some(Third),
    Some(Fourth),  Some(Fourth),  Some(Fourth),  Some(Fourth),  Some(Fourth),  Some(Fourth),  Some(Fourth),  Some(Fourth),  Some(Fourth),
    Some(Fifth),   Some(Fifth),   Some(Fifth),   Some(Fifth),   Some(Fifth),   Some(Fifth),   Some(Fifth),   Some(Fifth),   Some(Fifth),
    Some(Sixth),   Some(Sixth),   Some(Sixth),   Some(Sixth),   Some(Sixth),   Some(Sixth),   Some(Sixth),   Some(Sixth),   Some(Sixth),
    Some(Seventh), Some(Seventh), Some(Seventh), Some(Seventh), Some(Seventh), Some(Seventh), Some(Seventh), Some(Seventh), Some(Seventh),
    Some(Eighth),  Some(Eighth),  Some(Eighth),  Some(Eighth),  Some(Eighth),  Some(Eighth),  Some(Eighth),  Some(Eighth),  Some(Eighth),
    None,          None,          None,          None,          None,          None,          None,          None,          None,
];

#[rustfmt::skip]
const DEC_SQUARE: [Option<Square>; 90] = [
    None,     None,     None,     None,     None,     None,     None,     None,     None,
    Some(A1), Some(B1), Some(C1), Some(D1), Some(E1), Some(F1), Some(G1), Some(H1), None,
    Some(A2), Some(B2), Some(C2), Some(D2), Some(E2), Some(F2), Some(G2), Some(H2), None,
    Some(A3), Some(B3), Some(C3), Some(D3), Some(E3), Some(F3), Some(G3), Some(H3), None,
    Some(A4), Some(B4), Some(C4), Some(D4), Some(E4), Some(F4), Some(G4), Some(H4), None,
    Some(A5), Some(B5), Some(C5), Some(D5), Some(E5), Some(F5), Some(G5), Some(H5), None,
    Some(A6), Some(B6), Some(C6), Some(D6), Some(E6), Some(F6), Some(G6), Some(H6), None,
    Some(A7), Some(B7), Some(C7), Some(D7), Some(E7), Some(F7), Some(G7), Some(H7), None,
    Some(A8), Some(B8), Some(C8), Some(D8), Some(E8), Some(F8), Some(G8), Some(H8), None,
    None,     None,     None,     None,     None,     None,     None,     None,     None,
];

pub fn read_cdb_moves<B: Buf>(buf: &mut B) -> (Vec<(UciMove, i16)>, Option<u32>) {
    let mut scored_moves = Vec::new();
    let mut ply_from_root = None;
    while buf.has_remaining() {
        let src = buf.get_u8();
        let dst = buf.get_u8();
        let score = buf.get_i16();

        continue;

        if src & 0x7f == 0 && dst & 0x7f == 0 {
            ply_from_root = Some(u32::try_from(score).unwrap());
            continue;
        }

        let from = DEC_SQUARE[usize::from(src & 0x7f)].unwrap();
        let to_file = DEC_FILE[usize::from(dst & 0x7f)].unwrap();
        let to_rank = DEC_RANK[usize::from(dst & 0x7f)];

        let uci = if dst & 0x80 == 0 {
            UciMove::Normal {
                from,
                to: Square::from_coords(to_file, to_rank.unwrap()),
                promotion: None,
            }
        } else {
            UciMove::Normal {
                from,
                to: Square::from_coords(
                    to_file,
                    match from.rank() {
                        Rank::Seventh => Rank::Eighth,
                        Rank::Second => Rank::First,
                        r => panic!("invalid rank for promotion: {r}"),
                    },
                ),
                promotion: Some(match to_rank {
                    None => Role::Queen,
                    Some(Rank::First) => Role::Rook,
                    Some(Rank::Second) => Role::Bishop,
                    Some(Rank::Third) => Role::Knight,
                    r => panic!("invalid rank to encode promotion role: {r:?}"),
                }),
            }
        };

        scored_moves.push((uci, score));
    }

    (scored_moves, ply_from_root)
}
