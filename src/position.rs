use crate::{
    bitboard::Bitboard,
    board::Board,
    global::GlobalData,
    r#move::Move,
    shift::{self, Offset, Shift},
    types::*,
};

use std::ops::Deref;

#[derive(Clone)]
pub struct State {
    hash: u64,
    castling_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u32,

    material: [i16; Color::COUNT],
    square_score: i16,
}

pub struct UndoState {
    r#move: Move,
    capture: Option<Piece>,
}

#[derive(Clone)]
pub struct Position {
    board: Board,
    ply: u32,
    states: Vec<State>,
}

impl Position {
    pub const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn new() -> Self {
        Self::from_str(Self::STARTPOS)
    }
    
    pub fn from_str(fen: &str) -> Self {
        Self::parse(&fen.split(" ").collect::<Vec<_>>())
    }

    pub fn parse(fen: &[&str]) -> Self {
        let zobrist = GlobalData::get().zobrist();
        let mut board = Board::empty();
        let mut hash = 0;
        let mut material = [0, 0];

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
                    *piece.color().index_mut(&mut material) += piece.kind().value();
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
        let ply = fen[5].parse::<u32>().unwrap() * 2 - 2 + color as u32;

        let state = State {
            hash,
            castling_rights,
            en_passant,
            halfmove_clock,
            square_score: 0,
            material,
        };

        let mut position = Self {
            board,
            ply,
            states: vec![state],
        };

        position.states.last_mut().unwrap().square_score = position.square_scores();

        position
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

    pub fn ply(&self) -> u32 {
        self.ply
    }

    pub fn is_technical_draw(&self) -> bool {
        let (last, rest) = self.states.split_last().unwrap();

        if last.halfmove_clock >= 100 {
            return true;
        }

        for state in &rest[rest.len().saturating_sub(last.halfmove_clock as usize)..] {
            if state.hash == last.hash {
                return true;
            }
        }

        return false;
    }

    pub fn turn(&self) -> Color {
        match self.ply & 1 {
            0 => Color::White,
            1 => Color::Black,
            _ => unreachable!(),
        }
    }

    pub fn make_null(&mut self) {
        let data = GlobalData::get();
        let zobrist = data.zobrist();

        let mut state = self.state().clone();

        if let Some(ep) = state.en_passant {
            state.hash ^= zobrist.en_passant(ep.file());
        }

        state.hash ^= zobrist.color();
        state.en_passant = None;

        self.ply += 1;
        self.states.push(state);
    }

    pub fn unmake_null(&mut self) {
        self.states.pop();
        self.ply -= 1;
    }

    pub fn make(&mut self, r#move: Move) -> UndoState {
        let data = GlobalData::get();
        let zobrist = data.zobrist();
        let table = data.square();

        let mut state = self.state().clone();
        let piece = self.board.get(r#move.from()).unwrap();
        let mut capture = self.board.get(r#move.to());
        let mut phase = self.phase();

        self.board.set(r#move.from(), None);
        self.board.set(r#move.to(), Some(piece));

        state.hash ^= zobrist.piece(piece, r#move.from());
        state.hash ^= zobrist.piece(piece, r#move.to());

        state.square_score -= table.get(piece, r#move.from(), phase) * piece.color().sign();

        // Capture
        if let Some(captured) = capture {
            state.hash ^= zobrist.piece(captured, r#move.to());

            *captured.color().index_mut(&mut state.material) -= captured.kind().value();
        }

        // Promotion
        if let Some(promotion) = r#move.kind() {
            let promoted = Piece::new(piece.color(), promotion);

            self.board.set(r#move.to(), Some(promoted));

            state.hash ^= zobrist.piece(piece, r#move.to());
            state.hash ^= zobrist.piece(promoted, r#move.to());

            *piece.color().index_mut(&mut state.material) -= piece.kind().value();
            *piece.color().index_mut(&mut state.material) += promotion.value();
        }

        phase = self.phase();

        state.square_score += table.get(piece, r#move.to(), phase) * piece.color().sign();

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

                        state.square_score -=
                            table.get(rook, Square::H1, phase) * rook.color().sign();
                        state.square_score +=
                            table.get(rook, Square::F1, phase) * rook.color().sign();
                    }

                    if state.castling_rights.has(CastlingRights::WHITE_LONG)
                        && r#move.to() == Square::C1
                    {
                        self.board.set(Square::A1, None);
                        self.board.set(Square::D1, Some(rook));

                        state.hash ^= zobrist.piece(rook, Square::A1);
                        state.hash ^= zobrist.piece(rook, Square::D1);

                        state.square_score -=
                            table.get(rook, Square::A1, phase) * rook.color().sign();
                        state.square_score +=
                            table.get(rook, Square::D1, phase) * rook.color().sign();
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

                        state.square_score -=
                            table.get(rook, Square::H8, phase) * rook.color().sign();
                        state.square_score +=
                            table.get(rook, Square::F8, phase) * rook.color().sign();
                    }

                    if state.castling_rights.has(CastlingRights::BLACK_LONG)
                        && r#move.to() == Square::C8
                    {
                        self.board.set(Square::A8, None);
                        self.board.set(Square::D8, Some(rook));

                        state.hash ^= zobrist.piece(rook, Square::A8);
                        state.hash ^= zobrist.piece(rook, Square::D8);

                        state.square_score -=
                            table.get(rook, Square::A8, phase) * rook.color().sign();
                        state.square_score +=
                            table.get(rook, Square::D8, phase) * rook.color().sign();
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

            state.square_score -= table.get(captured, taken, phase) * captured.color().sign();
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
        (self.state().square_score + self.evaluate_side::<ConstWhite>()
            - self.evaluate_side::<ConstBlack>())
            * self.turn().sign()
    }

    pub fn evaluate_side<C: ConstColor>(&self) -> i16 {
        let mut score: i16 = 0;

        score += C::color().index(&self.state().material);
        score += self.evaluate_pawn_structure::<C>();
        //score += self.king_safety::<C>();
        //score += self.pawn_adjustment::<C>();

        score
    }

    pub fn evaluate_pawn_structure<C: ConstColor>(&self) -> i16 {
        const PROTECTED: i16 = 10;
        const ISOLATED: i16 = -25;
        const DOUBLED: i16 = -25;

        let mut score = 0;

        let bb = self.board.color_kind_bb(C::color(), Kind::Pawn);

        // Reward pawns protecting each other
        score += (bb & shift::pawn_attack::<C>(bb)).count() as i16 * PROTECTED;

        // Punish double pawns
        score += (bb & C::up().shift(bb)).count() as i16 * DOUBLED;

        // Punish isolated pawns
        let lines = shift::ray(shift::So, shift::ray(shift::No, bb, !Bitboard(0)), !Bitboard(0));
        let nb = Offset::<-1, 0>.shift(lines) | Offset::<1, 0>.shift(lines);
        let one = Into::<Bitboard>::into(Rank::_1) & (!nb & lines);

        score += one.count() as i16 * ISOLATED;

        score
    }

    pub fn king_safety<C: ConstColor>(&self) -> i16 {
        const SHIELD: i16 = 20;
        const STORM: i16 = -10;

        let mut score = 0;

        let sides = !(Into::<Bitboard>::into(File::D) | Into::<Bitboard>::into(File::E));
        let king = self.board.color_kind_bb(C::color(), Kind::King);
        let mut pawns = sides & self.board.color_kind_bb(C::color(), Kind::Pawn);
        let mut bb = king & sides;

        // Pawn shield
        bb |= Offset::<-1, 0>.shift(bb) | Offset::<1, 0>.shift(bb);
        bb |= C::up().shift(bb);
        bb |= C::up().shift(bb);

        score += (bb & pawns).count() as i16 * SHIELD;

        // Pawn storm
        pawns = sides & self.board.color_kind_bb(!C::color(), Kind::Pawn);
        bb |= C::up().shift(bb);
        bb |= C::up().shift(bb);

        score += (bb & pawns).count() as i16 * STORM;

        score
    }

    pub fn pawn_adjustment<C: ConstColor>(&self) -> i16 {
        // CP adjustment per pawn present
        const KNIGHT: i16 = 5;
        const ROOK: i16 = -5;
        const QUEEN: i16 = -10;

        let mut score = 0;

        let pawns = self.board.kind_bb(Kind::Pawn);
        let knights = self.board.color_kind_bb(C::color(), Kind::Knight);
        let rooks = self.board.color_kind_bb(C::color(), Kind::Rook);
        let queen = self.board.color_kind_bb(C::color(), Kind::Queen);

        score += pawns.count() as i16 * knights.count() as i16 * KNIGHT;
        score += pawns.count() as i16 * rooks.count() as i16 * ROOK;
        score += pawns.count() as i16 * queen.count() as i16 * QUEEN;

        score
    }

    // Used only in parse
    pub fn square_scores(&self) -> i16 {
        let phase = self.phase();
        let data = GlobalData::get();
        let table = data.square();
        let mut score = 0;

        for piece in Piece::iter() {
            let bb = self.board.piece_bb(piece);

            score += bb
                .map(|square| table.get(piece, square, phase) * piece.color().sign())
                .sum::<i16>();
        }

        score
    }

    pub fn all_material(&self) -> i16 {
        self.state().material.iter().sum::<i16>() - 2 * Kind::King.value()
    }

    pub fn phase(&self) -> Phase {
        const MIDGAME: i16 = 3700;
        const ENDGAME: i16 = 1000;

        match self.all_material() {
            MIDGAME.. => Phase::Opening,
            ENDGAME..MIDGAME => Phase::Middle,
            ..ENDGAME => Phase::Endgame,
        }
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

#[cfg(test)]
mod tests {
    use crate::gen::*;
    use rand::prelude::*;

    use super::*;

    const DEPTH: usize = 100;
    const FENS: [&str; 6] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ];

    #[test]
    fn fen() {
        for fen in FENS {
            let array: [&str; 6] = fen
                .split_whitespace()
                .collect::<Vec<&str>>()
                .try_into()
                .unwrap();

            assert!(fen == Position::parse(&array).fen());
        }
    }

    #[test]
    fn make_unmake_hash() {
        let mut position = Position::new();
        let mut rng = rand::rng();

        for _ in 0..DEPTH {
            let mut moves = MoveVec::new();

            generate_dyn::<true>(&mut moves, &position);

            let m = *moves.moves().choose(&mut rng).unwrap();

            println!("{m}");

            position.make(m);

            let f = position.fen();
            let fen: [&str; 6] = f
                .split_whitespace()
                .collect::<Vec<&str>>()
                .try_into()
                .unwrap();
            let cpy = Position::parse(&fen);

            assert!(position.hash() == cpy.hash());
        }
    }
}
