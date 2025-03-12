use crate::types::{Color, Kind, Piece, Square, Phase};

pub struct PieceSquareTable {
    values: [[[i16; 64]; Piece::COUNT]; Phase::COUNT],
}

impl PieceSquareTable {
    #[rustfmt::skip]
    const PAWN: [i16; 64] = [
         0,  0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        10, 10, 20, 30, 30, 20, 10, 10,
         5,  5, 10, 25, 25, 10,  5,  5,
         0,  0,  0, 20, 20,  0,  0,  0,
         5, -5,-10,  0,  0,-10, -5,  5,
         5, 10, 10,-20,-20, 10, 10,  5,
         0,  0,  0,  0,  0,  0,  0,  0
    ];
    
    #[rustfmt::skip]
    const PAWN_END: [i16; 64] = [
         0,  0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        40, 40, 40, 40, 40, 40, 40, 40,
        30, 30, 30, 30, 30, 30, 30, 30,  
        20, 20, 20, 20, 20, 20, 20, 20,  
        10, 10, 10, 10, 10, 10, 10, 10,  
         0,  0,  0,  0,  0,  0,  0,  0,  
         0,  0,  0,  0,  0,  0,  0,  0
    ];
    
    #[rustfmt::skip]
    const KNIGHT: [i16; 64] = [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ];

    #[rustfmt::skip]
    const BISHOP: [i16; 64] = [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ];

    #[rustfmt::skip]
    const ROOK: [i16; 64] = [
          0,  0,  0,  0,  0,  0,  0,  0,
          5, 10, 10, 10, 10, 10, 10,  5,
         -5,  0,  0,  0,  0,  0,  0, -5,
         -5,  0,  0,  0,  0,  0,  0, -5,
         -5,  0,  0,  0,  0,  0,  0, -5,
         -5,  0,  0,  0,  0,  0,  0, -5,
         -5,  0,  0,  0,  0,  0,  0, -5,
          0,  0,  0,  5,  5,  0,  0,  0
    ];

    #[rustfmt::skip]
    const QUEEN: [i16; 64] = [
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5,  5,  5,  5,  0,-10,
         -5,  0,  5,  5,  5,  5,  0, -5,
          0,  0,  5,  5,  5,  5,  0, -5,
        -10,  5,  5,  5,  5,  5,  0,-10,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20
    ];

    #[rustfmt::skip]
    const KING: [i16; 64] = [
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -20,-30,-30,-40,-40,-30,-30,-20,
        -10,-20,-20,-20,-20,-20,-20,-10,
         20, 20,  0,  0,  0,  0, 20, 20,
         20, 30, 10,  0,  0, 10, 30, 20
    ];

    #[rustfmt::skip]
    const KING_END: [i16; 64] = [
        -50,-40,-30,-20,-20,-30,-40,-50,
        -30,-20,-10,  0,  0,-10,-20,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-30,  0,  0,  0,  0,-30,-30,
        -50,-30,-30,-30,-30,-30,-30,-50
    ];

    pub fn new() -> Self {
        let mut white: [[[i16; 64]; Kind::COUNT]; Phase::COUNT] = [[[0; 64]; Kind::COUNT]; Phase::COUNT];
        let mut black: [[[i16; 64]; Kind::COUNT]; Phase::COUNT] = [[[0; 64]; Kind::COUNT]; Phase::COUNT];

        for phase in Phase::iter() {
            *Kind::Pawn.index_mut(phase.index_mut(&mut black)) = Self::PAWN;
            *Kind::Knight.index_mut(phase.index_mut(&mut black)) = Self::KNIGHT;
            *Kind::Bishop.index_mut(phase.index_mut(&mut black)) = Self::BISHOP;
            *Kind::Rook.index_mut(phase.index_mut(&mut black)) = Self::ROOK;
            *Kind::Queen.index_mut(phase.index_mut(&mut black)) = Self::QUEEN;
            *Kind::King.index_mut(phase.index_mut(&mut black)) = Self::KING;
        }

        *Kind::King.index_mut(Phase::Endgame.index_mut(&mut black)) = Self::KING_END;
        *Kind::Pawn.index_mut(Phase::Endgame.index_mut(&mut black)) = Self::PAWN_END;

        // Swap black value for white
        for phase in Phase::iter() {
            for kind in Kind::iter() {
                for square in Square::iter() {
                    let value: i16 = *square.index(kind.index(phase.index(&black)));

                    *square.r#for(Color::Black).index_mut(kind.index_mut(phase.index_mut(&mut white))) = value;
                }
            }
        }

        let mut table = Self { values: [[[0; 64]; Piece::COUNT]; Phase::COUNT] };

        for phase in Phase::iter() {
            for piece in Piece::iter() {
                *piece.index_mut(phase.index_mut(&mut table.values)) = *piece.kind().index(phase.index(&match piece.color() { Color::White => white, Color::Black => black }));
            }
        }

        table
    }

    pub fn get(&self, piece: Piece, square: Square, phase: Phase) -> i16 {
        *square.index(piece.index(phase.index(&self.values)))
    }
}

