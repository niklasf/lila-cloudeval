use bytes::Buf;
use shakmaty::{File, Rank};
use shakmaty::Square;
use shakmaty::uci::UciMove;

use Square::*;

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
        if src ==
        let value = buf.get_i16();
    }
    (scored_moves, ply_from_root)
}
