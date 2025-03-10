use std::arch::x86_64;

use crate::bitboard::Bitboard;
use crate::shift;
use crate::types::{File, Rank, Square};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Magic {
    index: usize,
    mask: Bitboard,
}

pub struct MagicTable {
    bitboards: Vec<Bitboard>,
    bishop: [Magic; 64],
    rook: [Magic; 64],
}

impl Magic {
    pub fn new(
        bitboards: &mut Vec<Bitboard>,
        square: Square,
        ray: fn(Bitboard, Bitboard) -> Bitboard,
    ) -> Self {
        let h =
            (Bitboard::from(File::A) | Bitboard::from(File::H)) & !Bitboard::from(square.file());

        let v =
            (Bitboard::from(Rank::_1) | Bitboard::from(Rank::_8)) & !Bitboard::from(square.rank());

        let mask = (ray)(square.into(), !Bitboard(0)) & !(h | v);
        let index = bitboards.len();

        let mut bitboard = Bitboard(0);

        loop {
            bitboards.push((ray)(square.into(), !bitboard));
            bitboard = (bitboard - mask) & mask;

            if bitboard == Bitboard(0) {
                break;
            }
        }

        Self { mask, index }
    }
}

impl MagicTable {
    pub fn new() -> Self {
        let mut bitboards = Vec::new();

        let mut bishop = [Magic {
            index: 0,
            mask: Bitboard(0),
        }; 64];

        let mut rook = [Magic {
            index: 0,
            mask: Bitboard(0),
        }; 64];

        for square in Square::iter() {
            *square.index_mut(&mut bishop) = Magic::new(&mut bitboards, square, shift::bishop_ray);
            *square.index_mut(&mut rook) = Magic::new(&mut bitboards, square, shift::rook_ray);
        }

        Self {
            bitboards,
            bishop,
            rook,
        }
    }

    pub fn bishop(&self, square: Square, occupied: Bitboard) -> Bitboard {
        let magic = square.index(&self.bishop);
        let index = unsafe { x86_64::_pext_u64(occupied.0, magic.mask.0) } as usize;

        self.bitboards[magic.index + index]
    }

    pub fn rook(&self, square: Square, occupied: Bitboard) -> Bitboard {
        let magic = square.index(&self.rook);
        let index = unsafe { x86_64::_pext_u64(occupied.0, magic.mask.0) } as usize;

        self.bitboards[magic.index + index]
    }
}
