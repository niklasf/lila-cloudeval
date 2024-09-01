use bytes::Buf;
use shakmaty::{uci::UciMove, File, Rank, Role, Square};
use File::*;
use Rank::*;

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

#[derive(Default)]
pub struct ScoredMoves {
    moves: Vec<(UciMove, i16)>,
    ply_from_root: Option<u32>,
}

impl ScoredMoves {
    pub fn new() -> ScoredMoves {
        ScoredMoves::default()
    }

    pub fn with_capacity(moves: usize) -> ScoredMoves {
        ScoredMoves {
            moves: Vec::with_capacity(moves),
            ply_from_root: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn ply_from_root(&self) -> Option<u32> {
        self.ply_from_root
    }

    pub fn read_cdb<B: Buf>(buf: &mut B) -> ScoredMoves {
        let mut res = ScoredMoves::with_capacity(buf.remaining());

        while buf.has_remaining() {
            let dst = usize::from(buf.get_u8());
            let src = usize::from(buf.get_u8());
            let score = buf.get_i16();

            if src == 0 && dst == 0 {
                res.ply_from_root = Some(u32::try_from(score).unwrap());
                continue;
            }

            let from = Square::from_coords(DEC_FILE[src].unwrap(), DEC_RANK[src].unwrap());
            let to_file = DEC_FILE[dst & 0x7f].unwrap();
            let to_rank = DEC_RANK[dst & 0x7f];

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

            res.moves.push((uci, score));
        }

        res
    }
}
