use std::num::NonZeroU32;

use bytes::{Buf, BufMut};
use shakmaty::{
    variant::Variant, Bitboard, ByColor, ByRole, Color, Piece, Rank, RemainingChecks, Role, Setup,
    Square,
};

pub struct VariantSetup {
    setup: Setup,
    variant: Variant,
}

fn write_nibbles<B: BufMut>(lo: u8, hi: u8, buf: &mut B) {
    debug_assert!(lo & 0xf == lo);
    debug_assert!(hi & 0xf == hi);
    buf.put_u8(lo | (hi << 4));
}

fn read_nibbles<B: Buf>(buf: &mut B) -> (u8, u8) {
    let byte = buf.get_u8();
    (byte & 0xf, byte >> 4)
}

fn write_leb128<B: BufMut>(mut n: u32, buf: &mut B) {
    while n > 127 {
        buf.put_u8(n as u8 | 128);
        n = n >> 7;
    }
    buf.put_u8(n as u8);
}

fn read_leb128<B: Buf>(buf: &mut B) -> u32 {
    let mut n = 0;
    let mut shift = 0;
    while buf.has_remaining() {
        let byte = buf.get_u8();
        n |= u32::from(byte & 127) << shift;
        shift += 7;
        if byte & 128 == 0 {
            break;
        }
    }
    n
}

fn read_byte<B: Buf>(buf: &mut B) -> u8 {
    if buf.has_remaining() {
        buf.get_u8()
    } else {
        0
    }
}

impl VariantSetup {
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_u64(self.setup.board.occupied().into());

        let pawn_pushed_to = self.setup.ep_square.map(|sq| sq.xor(Square::A2));
        let unmoved_rooks = self.setup.castling_rights;

        #[rustfmt::skip]
        let pack_piece = |sq, piece| -> u8 {
            match piece {
                Piece { role: Role::Pawn, .. } if pawn_pushed_to == Some(sq) => 12,
                Piece { role: Role::Pawn, color: Color::White } => 0,
                Piece { role: Role::Pawn, color: Color::Black } => 1,
                Piece { role: Role::Knight, color: Color::White } => 2,
                Piece { role: Role::Knight, color: Color::Black } => 3,
                Piece { role: Role::Bishop, color: Color::White } => 4,
                Piece { role: Role::Bishop, color: Color::Black } => 5,
                Piece { role: Role::Rook, color: Color::White } => if unmoved_rooks.contains(sq) { 13 } else { 6 },
                Piece { role: Role::Rook, color: Color::Black } => if unmoved_rooks.contains(sq) { 14 } else { 7 },
                Piece { role: Role::Queen, color: Color::White } => 8,
                Piece { role: Role::Queen, color: Color::Black } => 9,
                Piece { role: Role::King, color: Color::White } => 10,
                Piece { role: Role::King, color: Color::Black } => self.setup.turn.fold_wb(11, 15),
            }
        };

        let mut pieces = self.setup.board.clone().into_iter();
        while let Some((sq, piece)) = pieces.next() {
            write_nibbles(
                pack_piece(sq, piece),
                if let Some((sq, piece)) = pieces.next() {
                    pack_piece(sq, piece)
                } else {
                    0
                },
                buf,
            )
        }

        let ply = (u32::from(self.setup.fullmoves) - 1) * 2 + self.setup.turn.fold_wb(0, 1);
        let broken_turn = self.setup.turn.is_black()
            && (self.setup.board.by_role(Role::King) & self.setup.board.by_color(Color::Black))
                .is_empty();
        let variant_header = match self.variant {
            Variant::Chess => 0,
            Variant::Crazyhouse => 1,
            Variant::KingOfTheHill => 4,
            Variant::ThreeCheck => 5,
            Variant::Antichess => 6,
            Variant::Atomic => 7,
            Variant::Horde => 8,
            Variant::RacingKings => 9,
        };

        if self.setup.halfmoves > 0 || ply > 1 || broken_turn || variant_header != 0 {
            write_leb128(self.setup.halfmoves, buf);
        }

        if ply > 1 || broken_turn || variant_header != 0 {
            write_leb128(ply, buf);
        }

        if variant_header != 0 {
            buf.put_u8(variant_header);
        }

        match self.variant {
            Variant::ThreeCheck => {
                let remaining_checks = self.setup.remaining_checks.unwrap_or_default();
                write_nibbles(
                    remaining_checks.white.into(),
                    remaining_checks.black.into(),
                    buf,
                );
            }
            Variant::Crazyhouse => {
                let pockets = self.setup.pockets.unwrap_or_default();
                write_nibbles(pockets.white.pawn, pockets.black.pawn, buf);
                write_nibbles(pockets.white.knight, pockets.black.knight, buf);
                write_nibbles(pockets.white.bishop, pockets.black.bishop, buf);
                write_nibbles(pockets.white.rook, pockets.black.rook, buf);
                write_nibbles(pockets.white.queen, pockets.black.queen, buf);
                if self.setup.promoted.any() {
                    buf.put_u64(self.setup.promoted.into());
                }
            }
            _ => {}
        }
    }

    pub fn read<B: Buf>(buf: &mut B) -> VariantSetup {
        let mut setup = Setup::empty();

        #[rustfmt::skip]
        let mut unpack_piece = |sq: Square, packed: u8| {
            setup.board.set_piece_at(
                sq,
                match packed {
                    0 => Piece { color: Color::White, role: Role::Pawn },
                    1 => Piece { color: Color::Black, role: Role::Pawn },
                    2 => Piece { color: Color::White, role: Role::Knight },
                    3 => Piece { color: Color::Black, role: Role::Knight },
                    4 => Piece { color: Color::White, role: Role::Bishop },
                    5 => Piece { color: Color::Black, role: Role::Bishop },
                    6 => Piece { color: Color::White, role: Role::Rook },
                    7 => Piece { color: Color::Black, role: Role::Rook },
                    8 => Piece { color: Color::White, role: Role::Queen },
                    9 => Piece { color: Color::Black, role: Role::Queen },
                    10 => Piece { color: Color::White, role: Role::King },
                    11 => Piece { color: Color::Black, role: Role::King },
                    12 => {
                        setup.ep_square = Some(sq.xor(Square::A2));
                        Color::from_white(sq.rank() <= Rank::Fourth).pawn()
                    }
                    13 => {
                        setup.castling_rights.add(sq);
                        Piece { color: Color::White, role: Role::Rook }
                    }
                    14 => {
                        setup.castling_rights.add(sq);
                        Piece { color: Color::Black, role: Role::Rook }
                    }
                    15 => {
                        setup.turn = Color::Black;
                        Piece { color: Color::Black, role: Role::King }
                    }
                    _ => panic!("invalid packed piece: {packed} at {sq}"),
                },
            );
        };

        let mut occupied_iter = Bitboard(buf.get_u64()).into_iter();
        while let Some(sq) = occupied_iter.next() {
            let (lo, hi) = read_nibbles(buf);
            unpack_piece(sq, lo);
            if let Some(sq) = occupied_iter.next() {
                unpack_piece(sq, hi);
            }
        }

        setup.halfmoves = read_leb128(buf);
        let ply = read_leb128(buf);
        let variant = match read_byte(buf) {
            0 => Variant::Chess,
            1 => Variant::Crazyhouse,
            4 => Variant::KingOfTheHill,
            5 => Variant::ThreeCheck,
            6 => Variant::Antichess,
            7 => Variant::Atomic,
            8 => Variant::Horde,
            9 => Variant::RacingKings,
            n => panic!("invalid variant header: {n}"),
        };

        if ply % 2 == 1 {
            setup.turn = Color::Black;
        }

        setup.fullmoves = NonZeroU32::new(1 + ply / 2).expect("fullmoves");

        match variant {
            Variant::ThreeCheck => {
                let (lo, hi) = read_nibbles(buf);
                setup.remaining_checks = Some(ByColor {
                    white: RemainingChecks::new(lo.into()),
                    black: RemainingChecks::new(hi.into()),
                });
            }
            Variant::Crazyhouse => {
                let (wp, bp) = read_nibbles(buf);
                let (wn, bn) = read_nibbles(buf);
                let (wb, bb) = read_nibbles(buf);
                let (wr, br) = read_nibbles(buf);
                let (wq, bq) = read_nibbles(buf);
                setup.pockets = Some(ByColor {
                    white: ByRole {
                        pawn: wp,
                        knight: wn,
                        bishop: wb,
                        rook: wr,
                        queen: wq,
                        king: 0,
                    },
                    black: ByRole {
                        pawn: bp,
                        knight: bn,
                        bishop: bb,
                        rook: br,
                        queen: bq,
                        king: 0,
                    },
                });
            }
            _ => {}
        }

        VariantSetup { setup, variant }
    }
}
