use crate::bitboard::Bitboard;
use crate::types::{Color, Kind, Piece, Square};

// Pieces only
#[derive(Clone, Debug)]
pub struct Board {
    pieces: [Option<Piece>; 64],
    color_bb: [Bitboard; 2],
    kind_bb: [Bitboard; 6],
}

impl Board {
    pub fn empty() -> Self {
        Self {
            pieces: [None; 64],
            color_bb: [Bitboard(0); 2],
            kind_bb: [Bitboard(0); 6],
        }
    }

    pub fn get(&self, square: Square) -> Option<Piece> {
        *square.index(&self.pieces)
    }

    pub fn set(&mut self, square: Square, piece: Option<Piece>) {
        let bb: Bitboard = square.into();

        if let Some(piece) = self.get(square) {
            *piece.color().index_mut(&mut self.color_bb) &= !bb;
            *piece.kind().index_mut(&mut self.kind_bb) &= !bb;
        }

        *square.index_mut(&mut self.pieces) = piece;

        if let Some(piece) = piece {
            *piece.color().index_mut(&mut self.color_bb) |= bb;
            *piece.kind().index_mut(&mut self.kind_bb) |= bb;
        }
    }

    pub fn pieces(&self) -> [Option<Piece>; 64] {
        self.pieces
    }

    pub fn color_bb(&self, color: Color) -> Bitboard {
        *color.index(&self.color_bb)
    }

    pub fn kind_bb(&self, kind: Kind) -> Bitboard {
        *kind.index(&self.kind_bb)
    }
}
