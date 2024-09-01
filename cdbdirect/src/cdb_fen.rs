use std::mem;

use shakmaty::{Color, File, Piece, Rank, Role, Setup, Square};

#[derive(Default)]
pub struct Nibbles {
    bytes: Vec<u8>,
    half: bool,
}

impl Nibbles {
    pub fn new() -> Nibbles {
        Nibbles::default()
    }

    pub fn with_capacity(nibbles: usize) -> Nibbles {
        Nibbles {
            bytes: Vec::with_capacity((nibbles + 1) / 2),
            half: false,
        }
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
        self.half = false;
    }

    pub fn push_nibble(&mut self, nibble: u8) {
        debug_assert!(nibble & 0xf == nibble);

        if self.half {
            *self.bytes.last_mut().expect("non empty") |= nibble & 0xf;
        } else {
            self.bytes.push(nibble << 4);
        }
        self.half = !self.half;
    }

    pub fn push_byte(&mut self, byte: u8) {
        if self.half {
            *self.bytes.last_mut().expect("non empty") |= byte >> 4;
            self.bytes.push(byte << 4);
        } else {
            self.bytes.push(byte);
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

fn push_empty(nibbles: &mut Nibbles, empty: i32) {
    match empty {
        1 => nibbles.push_nibble(0x0),
        2 => nibbles.push_nibble(0x1),
        3 => nibbles.push_nibble(0x2),
        4 => nibbles.push_byte(0x80),
        5 => nibbles.push_byte(0x81),
        6 => nibbles.push_byte(0x82),
        7 => nibbles.push_byte(0x83),
        8 => nibbles.push_byte(0x84),
        _ => {}
    }
}

pub fn cdb_fen(setup: &Setup) -> Nibbles {
    let mut nibbles = Nibbles::with_capacity(2 + 10 + 1 + 1 + 1);
    push_cdb_fen(&mut nibbles, setup);
    nibbles
}

pub fn push_cdb_fen(nibbles: &mut Nibbles, setup: &Setup) {
    nibbles.push_byte(b'h');

    // Board
    for rank in Rank::ALL.into_iter().rev() {
        let mut empty = 0;
        for file in File::ALL {
            let square = Square::from_coords(file, rank);
            if let Some(piece) = setup.board.piece_at(square) {
                push_empty(nibbles, mem::take(&mut empty));

                nibbles.push_nibble(match piece {
                    Piece {
                        color: Color::Black,
                        role: Role::Pawn,
                    } => 0x3,
                    Piece {
                        color: Color::Black,
                        role: Role::Knight,
                    } => 0x4,
                    Piece {
                        color: Color::Black,
                        role: Role::Bishop,
                    } => 0x5,
                    Piece {
                        color: Color::Black,
                        role: Role::Rook,
                    } => 0x6,
                    Piece {
                        color: Color::Black,
                        role: Role::Queen,
                    } => 0x7,
                    Piece {
                        color: Color::Black,
                        role: Role::King,
                    } => 0x9,
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

        push_empty(nibbles, empty);
    }

    // Turn
    nibbles.push_nibble(setup.turn.fold_wb(0x0, 0x1));

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
                nibbles.push_nibble(color.fold_wb(0xb, 0xd)); // Q/q
            } else if Some(rook) == candidates.last() && king.map_or(false, |k| k < rook) {
                nibbles.push_nibble(color.fold_wb(0xa, 0xc)); // K/k
            } else {
                match color {
                    Color::Black => nibbles.push_nibble(0x1 + u8::from(rook.file())),
                    Color::White => {
                        nibbles.push_nibble(0xe);
                        nibbles.push_nibble(0x1 + u8::from(rook.file()));
                    }
                }
            }
            has_castling_rights = true;
        }
    }
    if !has_castling_rights {
        nibbles.push_nibble(0x0);
    }

    // Space
    nibbles.push_nibble(0x9);

    // Ep square
    if let Some(ep_square) = setup.ep_square {
        nibbles.push_nibble(0x1 + u8::from(ep_square.file()));
        nibbles.push_nibble(0x1 + u8::from(ep_square.rank()));
    }
}
