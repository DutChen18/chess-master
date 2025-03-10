use crate::{
    bitboard::Bitboard,
    shift::{self},
    types::{Color, ConstBlack, ConstWhite, Square},
};

pub struct AttackTable {
    pawn: [[Bitboard; 64]; 2],
    knight: [Bitboard; 64],
    king: [Bitboard; 64],
    between: [[Bitboard; 64]; 64],
    line: [[Bitboard; 64]; 64],
}

impl AttackTable {
    pub fn new() -> Self {
        let mut pawn = [[Bitboard(0); 64]; 2];
        let mut knight = [Bitboard(0); 64];
        let mut king = [Bitboard(0); 64];
        let mut between = [[Bitboard(0); 64]; 64];
        let mut line = [[Bitboard(0); 64]; 64];

        for square in Square::iter() {
            let bb = square.into();

            *square.index_mut(Color::White.index_mut(&mut pawn)) =
                shift::pawn_attack::<ConstWhite>(bb);
            *square.index_mut(Color::Black.index_mut(&mut pawn)) =
                shift::pawn_attack::<ConstBlack>(bb);
            *square.index_mut(&mut knight) = shift::knight_attack(bb);
            *square.index_mut(&mut king) = shift::king_attack(bb);

            for other in Square::iter() {
                let reachable_square = shift::rook_ray(bb, !Bitboard(0));
                let reachable_other = shift::rook_ray(other.into(), !Bitboard(0));

                if reachable_square.into_iter().any(|s| s == other)
                    && reachable_other.into_iter().any(|s| s == square)
                {
                    let mut all = reachable_square & reachable_other;

                    *square.index_mut(other.index_mut(&mut line)) = all;

                    for s in all.clone() {
                        // Remove squares not between points
                        if !((square < s && s < other) || (other < s && s < square)) {
                            all &= !Into::<Bitboard>::into(s);
                        }
                    }

                    *square.index_mut(other.index_mut(&mut between)) = all;
                }
                
                let reachable_square = shift::bishop_ray(bb, !Bitboard(0));
                let reachable_other = shift::bishop_ray(other.into(), !Bitboard(0));

                if reachable_square.into_iter().any(|s| s == other)
                    && reachable_other.into_iter().any(|s| s == square)
                {
                    let mut all = reachable_square & reachable_other;

                    *square.index_mut(other.index_mut(&mut line)) = all;

                    for s in all.clone() {
                        // Remove squares not between points
                        if !((square < s && s < other) || (other < s && s < square)) {
                            all &= !Into::<Bitboard>::into(s);
                        }
                    }

                    *square.index_mut(other.index_mut(&mut between)) = all;
                }
            }
        }

        AttackTable {
            pawn,
            knight,
            king,
            between,
            line,
        }
    }

    pub fn pawn(&self, square: Square, color: Color) -> Bitboard {
        *square.index(color.index(&self.pawn))
    }

    pub fn knight(&self, square: Square) -> Bitboard {
        *square.index(&self.knight)
    }

    pub fn king(&self, square: Square) -> Bitboard {
        *square.index(&self.king)
    }

    pub fn between(&self, from: Square, to: Square) -> Bitboard {
        *from.index(to.index(&self.between))
    }

    pub fn line(&self, from: Square, to: Square) -> Bitboard {
        *from.index(to.index(&self.line))
    }
}
