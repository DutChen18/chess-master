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
            material,
        };

        let position = Self {
            board,
            ply,
            states: vec![state],
        };

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

    pub fn captured_piece(&self, r#move: Move) -> Option<(Piece, Square)> {
        if let Some(piece) = self.get(r#move.to()) {
            Some((piece, r#move.to()))
        } else {
            let from_piece = self.get(r#move.from()).unwrap();

            if from_piece.kind() == Kind::Pawn && self.en_passant() == Some(r#move.to()) {
                let square = Square::new(r#move.to().file(), r#move.from().rank());

                Some((self.get(square).unwrap(), square))
            } else {
                None
            }
        }
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

        let mut state = self.state().clone();
        let piece = self.board.get(r#move.from()).unwrap();
        let mut capture = self.board.get(r#move.to());

        self.board.set(r#move.from(), None);
        self.board.set(r#move.to(), Some(piece));

        state.hash ^= zobrist.piece(piece, r#move.from());
        state.hash ^= zobrist.piece(piece, r#move.to());

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
        let global = GlobalData::get();

        let mut score = 0;

        score += self.evaluate_piece_square_table(global) * self.turn().sign();
        score += (self.evaluate_side::<ConstWhite>(global)
            - self.evaluate_side::<ConstBlack>(global))
            * self.turn().sign();

        score
    }

    pub fn evaluate_side<C: ConstColor>(&self, global: &GlobalData) -> i16 {
        let mut score: i16 = 0;

        score += *C::color().index(&self.state().material);
        score += self.pawn_structure::<C>();
        score += self.slider_mobility::<C>(global);
        score += self.bishop_pair::<C>();

        match self.phase() {
            Phase::Opening => (),
            Phase::Middle => (),
            Phase::Endgame => {
                if self.is_kingpawn_endgame() {
                    score += self.rule_of_the_square::<C>();
                }
            }
        }

        score
    }

    pub fn pawn_structure<C: ConstColor>(&self) -> i16 {
        const PROTECTED: i16 = 10;
        const DOUBLED: i16 = -20;
        const ISOLATED: i16 = -20;
        const PASSED: i16 = 20;

        let mut score = 0;

        let pawns = self.board.color_kind_bb(C::color(), Kind::Pawn);
        let pieces = self.board.color_bb(C::color()) & !self.board.kind_bb(Kind::King);

        // Pieces protected by pawns
        score += (pieces & shift::pawn_attack::<C>(pawns)).count() as i16 * PROTECTED;

        // Doubled pawns
        score += (pawns & C::up().shift(pawns)).count() as i16 * DOUBLED;

        // Isolated pawns
        let squashed = shift::squash(pawns);
        let nb = Offset::<-1, 0>.shift(squashed) | Offset::<1, 0>.shift(squashed);

        score += (!nb & Bitboard(0xFF)).count() as i16 * ISOLATED;

        let mut bb = self.board.color_kind_bb(!C::color(), Kind::Pawn);

        for _ in 0..6 {
            bb |= C::Opponent::up_left().shift(bb)
                | C::Opponent::up().shift(bb)
                | C::Opponent::up_right().shift(bb);
        }

        // Passed pawns
        for pawn in pawns & !bb {
            score += pawn.rank().r#for(C::color()) as i16 * PASSED;
        }

        score
    }

    pub fn slider_mobility<C: ConstColor>(&self, global: &GlobalData) -> i16 {
        const BISHOP_MOBILITY: i16 = 2;
        const ROOK_MOBILITY: i16 = 3;
        const BISHOP_BATTERY: i16 = 5;
        const ROOK_BATTERY: i16 = 10;

        let mut score = 0;

        let magic = global.magic();

        let bishops = self.bishop_queen_bb(C::color());
        let rooks = self.rook_queen_bb(C::color());
        let occupied = self.occupied_bb();

        for square in bishops {
            let bb = magic.bishop(square, occupied);

            score += bb.count() as i16 * BISHOP_MOBILITY;
            score += (bb & bishops).count() as i16 * BISHOP_BATTERY;
        }

        for square in rooks {
            let bb = magic.rook(square, occupied);

            score += bb.count() as i16 * ROOK_MOBILITY;
            score += (bb & rooks).count() as i16 * ROOK_BATTERY;
        }

        score
    }

    pub fn evaluate_piece_square_table(&self, global: &GlobalData) -> i16 {
        let phase = self.phase();
        let table = global.square();
        let mut score = 0;

        for piece in Piece::iter() {
            let bb = self.board.piece_bb(piece);

            score += bb
                .map(|square| table.get(piece, square, phase) * piece.color().sign())
                .sum::<i16>();
        }

        score
    }

    pub fn rule_of_the_square<C: ConstColor>(&self) -> i16 {
        const RULEOFTHESQUARE: i16 = 30;

        let mut score = 0;

        let mut pawns = self.board.color_kind_bb(C::color(), Kind::Pawn);
        let mut king = self.board.color_kind_bb(!C::color(), Kind::King);
        let top = Rank::_8.r#for(C::color()).into();

        for _ in 0..5 {
            pawns = C::up().shift(pawns);
            king |= shift::king_attack(king);

            if pawns & top & !king != Bitboard::EMPTY {
                score += RULEOFTHESQUARE;
                pawns &= !top;
            }
        }

        score
    }

    pub fn bishop_pair<C: ConstColor>(&self) -> i16 {
        const BISHOPPAIR: i16 = 50;

        let bishops = self.board.color_kind_bb(C::color(), Kind::Bishop);

        if bishops.count() == 2 {
            BISHOPPAIR
        } else {
            0
        }
    }

    pub fn king_safety<C: ConstColor>(&self) -> i16 {
        const SHIELD_0: i16 = 10;
        const SHIELD_1: i16 = 20;
        const SHIELD_2: i16 = 10;

        let pawns = self.board.color_kind_bb(C::color(), Kind::Pawn);
        let sides = !(Bitboard::from(File::D) | Bitboard::from(File::E) | Bitboard::from(File::F));
        let mut king = sides & self.board.color_kind_bb(C::color(), Kind::King);

        king |= Offset::<-1, 0>.shift(king) | Offset::<1, 0>.shift(king);

        (king & pawns).count() as i16 * SHIELD_0
            + (C::up().shift(king) & pawns).count() as i16 * SHIELD_1
            + (C::up_up().shift(king) & pawns).count() as i16 * SHIELD_2
    }

    pub fn is_kingpawn_endgame(&self) -> bool {
        self.board.kind_bb(Kind::King) | self.board.kind_bb(Kind::Pawn) == self.board.occupied_bb()
    }

    pub fn all_material(&self) -> i16 {
        self.state().material.iter().sum::<i16>() - 2 * Kind::King.value()
    }

    pub fn phase(&self) -> Phase {
        const MIDGAME: i16 = 3700;
        const ENDGAME: i16 = 1700;

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
