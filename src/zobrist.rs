use std::sync::LazyLock;

use rand::prelude::*;

use crate::types::{CastlingRights, File, Piece, Square};

pub struct Zobrist {
    piece: [[u64; 64]; 12],
    color: u64,
    castling_rights: [u64; 16],
    en_passant: [u64; 8],
}

impl Zobrist {
    pub fn get() -> &'static Self {
        &*ZOBRIST
    }

    pub fn piece(&self, piece: Piece, square: Square) -> u64 {
        *square.index(piece.index(&self.piece))
    }

    pub fn color(&self) -> u64 {
        self.color
    }

    pub fn castling_rights(&self, castling_rights: CastlingRights) -> u64 {
        *castling_rights.index(&self.castling_rights)
    }

    pub fn en_passant(&self, file: File) -> u64 {
        *file.index(&self.en_passant)
    }
}

static ZOBRIST: LazyLock<Zobrist> = LazyLock::new(|| {
    let mut rng = rand::rng();

    Zobrist {
        piece: rng.random(),
        color: rng.random(),
        castling_rights: rng.random(),
        en_passant: rng.random(),
    }
});
