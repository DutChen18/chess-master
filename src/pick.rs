use crate::{
    bitboard::Bitboard,
    board::Board,
    engine::Engine,
    gen::{self, MoveVec},
    global::GlobalData,
    r#move::Move,
    tt::Entry,
    types::{Color, ConstBlack, ConstColor, ConstWhite, Kind, Piece, Square},
};

pub struct Pick {
    moves: MoveVec,
    entry: Option<Entry>,
    capture_end: usize,
    index: usize,
}

impl Pick {
    pub fn new<const QUIET: bool>(engine: &mut Engine) -> Self {
        let hash = engine.position().hash();
        let mut moves = MoveVec::new();
        let mut entry = None;

        // TODO: incremental move generation

        gen::generate_dyn::<QUIET>(&mut moves, engine.position());

        if let Some(e) = engine.tt().probe(hash) {
            if let Some(index) = moves
                .moves()
                .iter()
                .position(|r#move| *r#move == e.r#move())
            {
                moves.moves_mut().swap(0, index);
                entry = Some(e);
            }
        }

        let tt_end = if entry.is_some() { 1 } else { 0 };
        let mut capture_end = tt_end;

        for i in tt_end..moves.moves().len() {
            let r#move = moves.moves()[i];

            if engine.position().get(r#move.to()).is_some() {
                moves.moves_mut().swap(i, capture_end);
                capture_end += 1;
            }
        }

        Pick {
            moves,
            entry,
            capture_end,
            index: 0,
        }
    }

    pub fn entry(&self) -> Option<Entry> {
        self.entry
    }

    pub fn next(&mut self, engine: &mut Engine) -> Option<(usize, Move)> {
        let tt_end = if self.entry.is_some() { 1 } else { 0 };

        if self.index == tt_end {
            sort_moves(
                engine,
                &mut self.moves.moves_mut()[tt_end..self.capture_end],
            );
        }

        if self.index == self.capture_end {
            sort_moves(engine, &mut self.moves.moves_mut()[self.capture_end..]);
        }

        if let Some(r#move) = self.moves.moves().get(self.index) {
            let index = self.index;

            self.index += 1;

            Some((index, *r#move))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.moves.moves().is_empty()
    }

    pub fn check(&self) -> bool {
        self.moves.check()
    }
}

fn lva<C: ConstColor>(
    global: &GlobalData,
    board: &Board,
    square: Square,
    occupied: Bitboard,
) -> Option<(Kind, Bitboard)> {
    let attack = global.attack();
    let magic = global.magic();

    let pawn_attack = attack.pawn(square, C::color());
    let pawn = board.color_kind_bb(C::opponent(), Kind::Pawn) & pawn_attack & occupied;

    if pawn != Bitboard(0) {
        return Some((Kind::Pawn, pawn));
    }

    let knight_attack = attack.knight(square);
    let knight = board.color_kind_bb(C::opponent(), Kind::Knight) & knight_attack & occupied;

    if knight != Bitboard(0) {
        return Some((Kind::Knight, knight));
    }

    let bishop_attack = magic.bishop(square, occupied);
    let bishop = board.color_kind_bb(C::opponent(), Kind::Bishop) & bishop_attack & occupied;

    if bishop != Bitboard(0) {
        return Some((Kind::Bishop, bishop));
    }

    let rook_attack = magic.rook(square, occupied);
    let rook = board.color_kind_bb(C::opponent(), Kind::Rook) & rook_attack & occupied;

    if rook != Bitboard(0) {
        return Some((Kind::Rook, rook));
    }

    let queen_attack = bishop_attack | rook_attack;
    let queen = board.color_kind_bb(C::opponent(), Kind::Queen) & queen_attack & occupied;

    if queen != Bitboard(0) {
        return Some((Kind::Queen, queen));
    }

    let king_attack = attack.king(square);
    let king = board.color_kind_bb(C::opponent(), Kind::King) & king_attack & occupied;

    if king != Bitboard(0) {
        return Some((Kind::King, king));
    }

    None
}

fn see<C: ConstColor>(global: &GlobalData, board: &Board, mut kind: Kind, r#move: Move) -> i32 {
    let mut occupied = board.occupied_bb();
    let mut stack = [0; 32];
    let mut depth = 1;

    if let Some(capture) = board.get(r#move.to()) {
        stack[0] = capture.kind().value() as i32;
    }

    while let Some((new_kind, bb)) = lva::<C>(global, board, r#move.to(), occupied) {
        occupied ^= bb & -bb;
        stack[depth] = kind.value() as i32 - stack[depth - 1];
        depth += 1;

        if let Some((new_new_kind, bb)) = lva::<C::Opponent>(global, board, r#move.to(), occupied) {
            occupied ^= bb & -bb;
            stack[depth] = new_kind.value() as i32 - stack[depth - 1];
            depth += 1;
            kind = new_new_kind;
        }
    }

    while depth > 1 {
        stack[depth - 2] = i32::min(stack[depth - 2], -stack[depth - 1]);
        depth -= 1;
    }

    stack[0]
}

fn see_dyn(global: &GlobalData, board: &Board, piece: Piece, r#move: Move) -> i32 {
    match piece.color() {
        Color::White => see::<ConstWhite>(global, board, piece.kind(), r#move),
        Color::Black => see::<ConstBlack>(global, board, piece.kind(), r#move),
    }
}

fn sort_moves(engine: &mut Engine, moves: &mut [Move]) {
    let global = GlobalData::get();
    let piece_square = global.square();
    let phase = engine.position().phase();

    moves.sort_by_cached_key(|r#move| {
        let piece = engine.position().get(r#move.from()).unwrap();

        -(see_dyn(global, engine.position(), piece, *r#move)
            + (piece_square.get(piece, r#move.to(), phase)
                - piece_square.get(piece, r#move.from(), phase)) as i32)

        // let piece = engine.position().get(r#move.from()).unwrap();

        // if let Some(capture) = engine.position().get(r#move.to()) {
        //     const VALUE: [i32; 6] = [0, 1, 2, 3, 4, 5];

        //     return *piece.kind().index(&VALUE) + (5 - *capture.kind().index(&VALUE)) * 6;
        // }

        // 1000000
        //     + (piece_square.get(piece, r#move.from(), phase)
        //         - piece_square.get(piece, r#move.to(), phase)) as i32

        // TODO: killer moves, history heuristic, SEE
    });
}
