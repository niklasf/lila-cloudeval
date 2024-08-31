use shakmaty::fen::Fen;
use shakmaty::Color;
use shakmaty::File;
use shakmaty::Piece;
use shakmaty::Rank;
use shakmaty::Role;
use shakmaty::Setup;
use shakmaty::Square;
use std::mem;
use serde_with::{DisplayFromStr, serde_as};
use serde::Deserialize;

fn push_empty(hex_fen: &mut String, empty: i32) {
    match empty {
        1 => hex_fen.push('0'),
        2 => hex_fen.push('1'),
        3 => hex_fen.push('2'),
        4 => {
            hex_fen.push('8');
            hex_fen.push('0');
        }
        5 => {
            hex_fen.push('8');
            hex_fen.push('1');
        }
        6 => {
            hex_fen.push('8');
            hex_fen.push('2');
        }
        7 => {
            hex_fen.push('8');
            hex_fen.push('3');
        }
        8 => {
            hex_fen.push('8');
            hex_fen.push('4');
        }
        _ => {}
    }
}

fn hex_fen(setup: &Setup) -> String {
    let mut hex_fen = String::new();

    // Board
    for rank in Rank::ALL.into_iter().rev() {
        let mut empty = 0;
        for file in File::ALL {
            let square = Square::from_coords(file, rank);
            if let Some(piece) = setup.board.piece_at(square) {
                push_empty(&mut hex_fen, mem::take(&mut empty));

                hex_fen.push(match piece {
                    Piece {
                        color: Color::Black,
                        role: Role::Pawn,
                    } => '3',
                    Piece {
                        color: Color::Black,
                        role: Role::Knight,
                    } => '4',
                    Piece {
                        color: Color::Black,
                        role: Role::Bishop,
                    } => '5',
                    Piece {
                        color: Color::Black,
                        role: Role::Rook,
                    } => '6',
                    Piece {
                        color: Color::Black,
                        role: Role::Queen,
                    } => '7',
                    Piece {
                        color: Color::Black,
                        role: Role::King,
                    } => '9',
                    Piece {
                        color: Color::White,
                        role: Role::Pawn,
                    } => 'a',
                    Piece {
                        color: Color::White,
                        role: Role::Knight,
                    } => 'b',
                    Piece {
                        color: Color::White,
                        role: Role::Bishop,
                    } => 'c',
                    Piece {
                        color: Color::White,
                        role: Role::Rook,
                    } => 'd',
                    Piece {
                        color: Color::White,
                        role: Role::Queen,
                    } => 'e',
                    Piece {
                        color: Color::White,
                        role: Role::King,
                    } => 'f',
                });
            } else {
                empty += 1;
            }
        }

        push_empty(&mut hex_fen, empty);
    }

    // Turn
    hex_fen.push(setup.turn.fold_wb('0', '1'));

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
                hex_fen.push(color.fold_wb('b', 'd')); // Q/q
            } else if Some(rook) == candidates.last() && king.map_or(false, |k| k < rook) {
                hex_fen.push(color.fold_wb('a', 'c')); // K/k
            } else {
                match color {
                    Color::Black => hex_fen.push(match rook.file() {
                        File::A => '1',
                        File::B => '2',
                        File::C => '3',
                        File::D => '4',
                        File::E => '5',
                        File::F => '6',
                        File::G => '7',
                        File::H => '8',
                    }),
                    Color::White => {
                        hex_fen.push('e');
                        hex_fen.push(match rook.file() {
                            File::A => 'a',
                            File::B => 'b',
                            File::C => 'c',
                            File::D => 'd',
                            File::E => 'e',
                            File::F => 'f',
                            File::G => 'g',
                            File::H => 'h',
                        })
                    }
                }
            }
            has_castling_rights = true;
        }
    }
    if !has_castling_rights {
        hex_fen.push('0');
    }

    // Ep square
    if let Some(ep_square) = setup.ep_square {
        hex_fen.push(match ep_square.file() {
            File::A => '1',
            File::B => '2',
            File::C => '3',
            File::D => '4',
            File::E => '5',
            File::F => '6',
            File::G => '7',
            File::H => '8',
        });
        hex_fen.push('e');
        hex_fen.push(match ep_square.rank() {
            Rank::First => 'a',
            Rank::Second => 'b',
            Rank::Third => 'c',
            Rank::Fourth => 'd',
            Rank::Fifth => 'e',
            Rank::Sixth => 'f',
            Rank::Seventh => 'g',
            Rank::Eighth => 'h',
        });
    } else {
        hex_fen.push('9');
    }

    if hex_fen.len() % 2 == 1 {
        hex_fen.push('0');
    }

    hex_fen
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
        assert_eq!(hex_fen(record.fen.as_setup()), record.cdb_fen, "line {}", i + 1);
    }
}
