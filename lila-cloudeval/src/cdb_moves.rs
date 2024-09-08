use std::cmp::Reverse;

use bytes::Buf;
use shakmaty::{uci::UciMove, File, Rank, Role, Square};
use File::*;
use Rank::*;

use crate::cdb_fen::NaturalOrder;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct RelativeScore(pub i16);

#[derive(Debug)]
pub struct ScoredMove {
    pub uci: UciMove,
    pub score: RelativeScore,
}

#[derive(Default, Debug)]
pub struct ScoredMoves {
    moves: Vec<ScoredMove>,
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

    pub fn moves(&self) -> &[ScoredMove] {
        &self.moves
    }

    pub fn num_good_moves(&self) -> usize {
        self.moves
            .iter()
            .filter(|entry| entry.score >= RelativeScore(0))
            .count()
    }

    pub fn clear(&mut self) {
        self.moves.clear();
        self.ply_from_root = None;
    }

    pub fn into_sorted(mut self) -> SortedScoredMoves {
        self.moves.sort_by_key(|entry| Reverse(entry.score));
        SortedScoredMoves(self)
    }

    pub fn read_cdb<B: Buf>(buf: &mut B, natural_order: NaturalOrder) -> ScoredMoves {
        let mut res = ScoredMoves::with_capacity(buf.remaining() / 4);
        res.extend_from_cdb(buf, natural_order);
        res
    }

    pub fn extend_from_cdb<B: Buf>(&mut self, buf: &mut B, natural_order: NaturalOrder) {
        while buf.has_remaining() {
            let dst = usize::from(buf.get_u8());
            let src = usize::from(buf.get_u8());
            let score = buf.get_i16_le();

            if src == 0 && dst == 0 {
                self.ply_from_root = Some(u32::try_from(score).unwrap());
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

            self.moves.push(ScoredMove {
                uci: match natural_order {
                    NaturalOrder::Same => uci,
                    NaturalOrder::Mirror => uci.to_mirrored(),
                },
                score: RelativeScore(score.checked_neg().expect("negated score")),
            });
        }
    }
}

pub struct SortedScoredMoves(pub ScoredMoves);

impl SortedScoredMoves {
    pub fn ply_from_root(&self) -> Option<u32> {
        self.0.ply_from_root()
    }

    pub fn moves(&self) -> &[ScoredMove] {
        self.0.moves()
    }

    pub fn into_moves(self) -> Vec<ScoredMove> {
        self.0.moves
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn into_best_moves(self, at_least: usize) -> SortedScoredMoves {
        let moves = if at_least < 1 {
            vec![]
        } else {
            let min_score = self
                .moves()
                .get(at_least - 1)
                .map_or(RelativeScore(i16::MIN), |entry| entry.score);

            self.0
                .moves
                .into_iter()
                .take_while(|entry| entry.score >= min_score)
                .collect()
        };

        SortedScoredMoves(ScoredMoves {
            moves,
            ply_from_root: self.0.ply_from_root,
        })
    }
}
