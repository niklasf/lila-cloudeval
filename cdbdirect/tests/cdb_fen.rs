use std::mem;

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, Color, File, Piece, Rank, Role, Setup, Square};

fn push_empty(nibbles: &mut Vec<u8>, empty: i32) {
    match empty {
        1 => nibbles.push(0),
        2 => nibbles.push(1),
        3 => nibbles.push(2),
        4 => {
            nibbles.push(8);
            nibbles.push(0);
        }
        5 => {
            nibbles.push(8);
            nibbles.push(1);
        }
        6 => {
            nibbles.push(8);
            nibbles.push(2);
        }
        7 => {
            nibbles.push(8);
            nibbles.push(3);
        }
        8 => {
            nibbles.push(8);
            nibbles.push(4);
        }
        _ => {}
    }
}

fn nibble_fen(setup: &Setup) -> Vec<u8> {
    let mut nibbles = Vec::new();

    // Board
    for rank in Rank::ALL.into_iter().rev() {
        let mut empty = 0;
        for file in File::ALL {
            let square = Square::from_coords(file, rank);
            if let Some(piece) = setup.board.piece_at(square) {
                push_empty(&mut nibbles, mem::take(&mut empty));

                nibbles.push(match piece {
                    Piece {
                        color: Color::Black,
                        role: Role::Pawn,
                    } => 3,
                    Piece {
                        color: Color::Black,
                        role: Role::Knight,
                    } => 4,
                    Piece {
                        color: Color::Black,
                        role: Role::Bishop,
                    } => 5,
                    Piece {
                        color: Color::Black,
                        role: Role::Rook,
                    } => 6,
                    Piece {
                        color: Color::Black,
                        role: Role::Queen,
                    } => 7,
                    Piece {
                        color: Color::Black,
                        role: Role::King,
                    } => 9,
                    Piece {
                        color: Color::White,
                        role: Role::Pawn,
                    } => 0xa,
                    Piece {
                        color: Color::White,
                        role: Role::Knight,
                    } => 0xb,
                    Piece {
                        color: Color::White,
                        role: Role::Bishop,
                    } => 0xc,
                    Piece {
                        color: Color::White,
                        role: Role::Rook,
                    } => 0xd,
                    Piece {
                        color: Color::White,
                        role: Role::Queen,
                    } => 0xe,
                    Piece {
                        color: Color::White,
                        role: Role::King,
                    } => 0xf,
                });
            } else {
                empty += 1;
            }
        }

        push_empty(&mut nibbles, empty);
    }

    // Turn
    nibbles.push(setup.turn.fold_wb(0, 1));

    // Castling rights
    let mut has_castling_rights = false;
    for color in Color::ALL {
        let king = setup
            .board
            .king_of(color)
            .filter(|k| k.rank() == color.backrank());
        let candidates = setup.board.by_piece(color.rook()) & color.backrank();
        for rook in (setup.castling_rights & color.backrank()).into_iter().rev() {
            if Some(rook) == candidates.first() && king.map_or(false, |k| rook < k) {
                nibbles.push(color.fold_wb(0xb, 0xd)); // Q/q
            } else if Some(rook) == candidates.last() && king.map_or(false, |k| k < rook) {
                nibbles.push(color.fold_wb(0xa, 0xc)); // K/k
            } else {
                match color {
                    Color::Black => nibbles.push(u8::from(rook.file()) + 1),
                    Color::White => {
                        nibbles.push(0xe);
                        nibbles.push(u8::from(rook.file()) + 0xa);
                    }
                }
            }
            has_castling_rights = true;
        }
    }
    if !has_castling_rights {
        nibbles.push(0);
    }

    // Space
    nibbles.push(9);

    // Ep square
    if let Some(ep_square) = setup.ep_square {
        nibbles.push(u8::from(ep_square.file()) + 1);
        nibbles.push(u8::from(ep_square.rank()) + 1);
    }

    if nibbles.len() % 2 == 1 {
        nibbles.push(0);
    }

    nibbles
}

fn hex_fen(setup: &Setup) -> String {
    let mut hex = String::new();
    for nibble in nibble_fen(setup) {
        hex.push(char::from_digit(u32::from(nibble), 16).unwrap());
    }
    hex
}

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
    let mut reader = csv::Reader::from_path("tests/cdb_fen.csv").expect("reader");
    for (i, record) in reader.deserialize().enumerate() {
        let record: Record = record.expect("record");

        assert_eq!(
            hex_fen(record.fen.as_setup()),
            record.cdb_fen,
            "line {}: cdb_fen mismatch for {}",
            i + 2,
            record.fen
        );

        assert_eq!(
            hex_fen(&bw(record.fen.as_setup().clone())),
            record.cdb_fen_bw,
            "line {}: cdb_fen_bw mismatch for {}",
            i + 2,
            record.fen
        );
    }
}
