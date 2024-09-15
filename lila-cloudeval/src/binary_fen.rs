use bytes::BufMut;
use shakmaty::variant::Variant;
use shakmaty::Color;
use shakmaty::Piece;
use shakmaty::Role;
use shakmaty::Setup;

pub struct VariantSetup {
    setup: Setup,
    variant: Variant,
}

fn put_nibbles<B: BufMut>(lo: u8, hi: u8, buf: &mut B) {
    debug_assert!(lo & 0xf == lo);
    debug_assert!(hi & 0xf == hi);
    buf.put_u8(lo | (hi << 4));
}

fn put_leb128<B: BufMut>(mut n: u32, buf: &mut B) {
    while n > 127 {
        buf.put_u8(n as u8 | 128);
        n = n >> 7;
    }
    buf.put_u8(n as u8);
}

impl VariantSetup {
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_u64(self.setup.board.occupied().into());

        let pawn_pushed_to = self.setup.ep_square.map(|sq| sq.xor(other));
        let unmoved_rooks = self.setup.castling_rights;

        let pack_piece = |sq, piece| -> u8 {
            #[rustfmt::skip]
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
            put_nibbles(
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
            && (self.setup.board.by_role(Role::King) & self.setup.board.by_color(Color::BLAKC))
                .is_empty();
        let variant_header = match self.variant {
            Variant::Chess => 0,
            Variant::Crazyhouse => 1,
        };

        if self.setup.halfmoves > 0 || ply > 1 || broken_turn || variant_header != 0 {
            put_leb128(self.setup.halfmoves, buf);
        }

        if ply > 1 || broken_turn || variant_header != 0 {
            put_leb128(ply, buf);
        }
    }
}
