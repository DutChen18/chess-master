use crate::r#move::Move;
use crate::types::{CastlingRights, Color, File, Kind, Piece, Rank, Square};
use crate::{board::Board, global::GlobalData};

use std::ops::Deref;

#[derive(Clone)]
pub struct State {
    hash: u64,
    castling_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u32,
}

pub struct UndoState {
    r#move: Move,
    capture: Option<Piece>,
}

#[derive(Clone)]
pub struct Position {
    board: Board,
    ply: usize,
    states: Vec<State>,
}

impl Position {
    pub const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn new() -> Self {
        Self::parse(&Self::STARTPOS.split(" ").collect::<Vec<_>>())
    }

    pub fn parse(fen: &[&str]) -> Self {
        let zobrist = GlobalData::get().zobrist();
        let mut board = Board::empty();
        let mut hash = 0;

        for (rank, string) in Rank::iter().rev().zip(fen[0].split("/")) {
            let mut files = File::iter();

            for ch in string.chars() {
                if ch.is_ascii_digit() {
                    for _ in '0'..ch {
                        files.next();
                    }
                } else {
                    let square = Square::new(files.next().unwrap(), rank);
                    let piece = Piece::from_char(ch);

                    board.set(square, Some(piece));
                    hash ^= zobrist.piece(piece, square);
                }
            }
        }

        let color = Color::from_str(fen[1]);

        if color == Color::Black {
            hash ^= zobrist.color();
        }

        let castling_rights = CastlingRights::from_str(fen[2]);

        hash ^= zobrist.castling_rights(castling_rights);

        let en_passant = if fen[3] == "-" {
            None
        } else {
            let square = Square::from_str(fen[3]);

            hash ^= zobrist.en_passant(square.file());

            Some(square)
        };

        let halfmove_clock = fen[4].parse().unwrap();
        let ply = fen[5].parse::<usize>().unwrap() * 2 - 2 + color as usize;

        let state = State {
            hash,
            castling_rights,
            en_passant,
            halfmove_clock,
        };

        Self {
            board,
            ply,
            states: vec![state],
        }
    }

    pub fn fen(&self) -> String {
        let mut fen = String::new();
        let mut empty = 0;

        for rank in Rank::iter().rev() {
            for file in File::iter() {
                if let Some(piece) = self.board.get(Square::new(file, rank)) {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }

                    fen.push_str(&piece.to_string());
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
                empty = 0;
            }

            if rank != Rank::_1 {
                fen.push_str("/");
            }
        }

        fen.push(' ');
        fen.push_str(match self.turn() {
            Color::White => "w",
            Color::Black => "b",
        });
        fen.push(' ');
        fen.push_str(&format!("{}", self.castling_rights()));
        fen.push(' ');

        match self.en_passant() {
            Some(square) => fen.push_str(&format!("{}", square)),
            None => fen.push('-'),
        }

        fen.push_str(&format!(
            " {} {}",
            self.state().halfmove_clock,
            self.ply / 2 + 1
        ));

        fen
    }

    pub fn king_square(&self, color: Color) -> Square {
        self.color_kind_bb(color, Kind::King).square().unwrap()
    }

    pub fn state(&self) -> &State {
        self.states.last().unwrap()
    }

    pub fn castling_rights(&self) -> CastlingRights {
        self.state().castling_rights
    }

    pub fn en_passant(&self) -> Option<Square> {
        self.state().en_passant
    }

    pub fn hash(&self) -> u64 {
        self.state().hash
    }

    pub fn turn(&self) -> Color {
        match self.ply & 1 {
            0 => Color::White,
            1 => Color::Black,
            _ => unreachable!(),
        }
    }

    pub fn make(&mut self, r#move: Move) -> UndoState {
        let zobrist = GlobalData::get().zobrist();
        let mut state = self.states.last().unwrap().clone();
        let piece = self.board.get(r#move.from()).unwrap();
        let mut capture = self.board.get(r#move.to());

        self.board.set(r#move.from(), None);
        self.board.set(r#move.to(), Some(piece));

        state.hash ^= zobrist.piece(piece, r#move.from());
        state.hash ^= zobrist.piece(piece, r#move.to());

        // Capture
        if let Some(captured) = capture {
            state.hash ^= zobrist.piece(captured, r#move.to());
        }

        // Promotion
        if let Some(promotion) = r#move.kind() {
            let promoted = Piece::new(piece.color(), promotion);

            self.board.set(r#move.to(), Some(promoted));

            state.hash ^= zobrist.piece(piece, r#move.to());
            state.hash ^= zobrist.piece(promoted, r#move.to());
        }

        // Move rook when castling
        if piece.kind() == Kind::King {
            let rook = Piece::new(piece.color(), Kind::Rook);

            match self.turn() {
                Color::White => {
                    if state.castling_rights.has(CastlingRights::WHITE_SHORT)
                        && r#move.to() == Square::G1
                    {
                        self.board.set(Square::H1, None);
                        self.board.set(Square::F1, Some(rook));

                        state.hash ^= zobrist.piece(rook, Square::H1);
                        state.hash ^= zobrist.piece(rook, Square::F1);
                    }

                    if state.castling_rights.has(CastlingRights::WHITE_LONG)
                        && r#move.to() == Square::C1
                    {
                        self.board.set(Square::A1, None);
                        self.board.set(Square::D1, Some(rook));

                        state.hash ^= zobrist.piece(rook, Square::A1);
                        state.hash ^= zobrist.piece(rook, Square::D1);
                    }
                }
                Color::Black => {
                    if state.castling_rights.has(CastlingRights::BLACK_SHORT)
                        && r#move.to() == Square::G8
                    {
                        self.board.set(Square::H8, None);
                        self.board.set(Square::F8, Some(rook));

                        state.hash ^= zobrist.piece(rook, Square::H8);
                        state.hash ^= zobrist.piece(rook, Square::F8);
                    }

                    if state.castling_rights.has(CastlingRights::BLACK_LONG)
                        && r#move.to() == Square::C8
                    {
                        self.board.set(Square::A8, None);
                        self.board.set(Square::D8, Some(rook));

                        state.hash ^= zobrist.piece(rook, Square::A8);
                        state.hash ^= zobrist.piece(rook, Square::D8);
                    }
                }
            }
        }

        // Take pawn upon en Passant
        if piece.kind() == Kind::Pawn && Some(r#move.to()) == state.en_passant {
            let taken = Square::new(r#move.to().file(), r#move.from().rank());
            let captured = self.board.get(taken).unwrap();

            self.board.set(taken, None);

            capture = Some(Piece::new(!self.turn(), Kind::Pawn));

            state.hash ^= zobrist.piece(captured, taken);
        }

        if let Some(ep) = state.en_passant {
            state.hash ^= zobrist.en_passant(ep.file());
        }

        state.halfmove_clock += 1;
        state.en_passant = None;

        // Halfmoves
        if capture != None || piece.kind() == Kind::Pawn {
            state.halfmove_clock = 0;
        }

        // Set en passant
        if piece.kind() == Kind::Pawn {
            match piece.color() {
                Color::White => {
                    if r#move.from().rank() == Rank::_2 && r#move.to().rank() == Rank::_4 {
                        let ep = Square::new(r#move.from().file(), Rank::_3);

                        state.en_passant = Some(ep);
                        state.hash ^= zobrist.en_passant(ep.file());
                    }
                }
                Color::Black => {
                    if r#move.from().rank() == Rank::_7 && r#move.to().rank() == Rank::_5 {
                        let ep = Square::new(r#move.from().file(), Rank::_6);

                        state.en_passant = Some(ep);
                        state.hash ^= zobrist.en_passant(ep.file());
                    }
                }
            }
        }

        // Set castling
        state.castling_rights &= castling_rights_mask(r#move.from());
        state.castling_rights &= castling_rights_mask(r#move.to());

        self.ply += 1;

        state.hash ^= zobrist.color();
        state.hash ^= zobrist.castling_rights(self.states.last().unwrap().castling_rights);
        state.hash ^= zobrist.castling_rights(state.castling_rights);

        self.states.push(state);

        UndoState { capture, r#move }
    }

    pub fn unmake(&mut self, undo_state: UndoState) {
        self.states.pop();

        let m = undo_state.r#move;
        let piece = self.board.get(m.to()).unwrap();
        let state = self.states.last().unwrap();

        self.ply -= 1;
        self.board.set(m.from(), Some(piece));
        self.board.set(m.to(), None);

        // Reset captured piece
        if let Some(capture) = undo_state.capture {
            let mut taken_square = m.to();

            if piece.kind() == Kind::Pawn && Some(m.to()) == state.en_passant {
                taken_square = Square::new(m.to().file(), m.from().rank());
            }

            self.board.set(taken_square, Some(capture));
        }

        // Undo promotion
        if m.kind() != None {
            self.board
                .set(m.from(), Some(Piece::new(piece.color(), Kind::Pawn)));
        }

        // Corner rook if undoing castling
        if piece.kind() == Kind::King {
            let rook = Some(Piece::new(piece.color(), Kind::Rook));

            match self.turn() {
                Color::White => {
                    if m.from() == Square::E1 && m.to() == Square::G1 {
                        self.board.set(Square::H1, rook);
                        self.board.set(Square::F1, None);
                    }

                    if m.from() == Square::E1 && m.to() == Square::C1 {
                        self.board.set(Square::A1, rook);
                        self.board.set(Square::D1, None);
                    }
                }
                Color::Black => {
                    if m.from() == Square::E8 && m.to() == Square::G8 {
                        self.board.set(Square::H8, rook);
                        self.board.set(Square::F8, None);
                    }

                    if m.from() == Square::E8 && m.to() == Square::C8 {
                        self.board.set(Square::A8, rook);
                        self.board.set(Square::D8, None);
                    }
                }
            }
        }
    }

    // Relative to side
    pub fn evaluate(&self) -> i16 {
        let mut scores: [i16; Color::COUNT] = [0; Color::COUNT];
        let data = GlobalData::get();
        let table = data.piece_square();

        for piece in Piece::iter() {
            let score: &mut i16 = piece.color().index_mut(&mut scores);

            // Material score
            //*score += self.board.piece_bb(piece).count() as i16 * piece.kind().value();

            // Square score TODO
            *score += self.board.piece_bb(piece).map(|square| table.get(piece, square)).sum::<i16>();
        }

        self.turn().index(&scores) - (!self.turn()).index(&scores)
    }
}

impl Deref for Position {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}

fn castling_rights_mask(square: Square) -> CastlingRights {
    match square {
        Square::H1 => !CastlingRights::WHITE_SHORT,
        Square::A1 => !CastlingRights::WHITE_LONG,
        Square::E1 => !CastlingRights::WHITE,
        Square::H8 => !CastlingRights::BLACK_SHORT,
        Square::A8 => !CastlingRights::BLACK_LONG,
        Square::E8 => !CastlingRights::BLACK,
        _ => CastlingRights::ALL,
    }
}
