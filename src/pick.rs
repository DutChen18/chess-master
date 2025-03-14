use std::mem::{self, MaybeUninit};

use crate::{
    bitboard::Bitboard,
    board::Board,
    engine::Engine,
    gen::{Generator, MoveList},
    global::GlobalData,
    position::Position,
    r#move::Move,
    tt::Entry,
    types::{Color, ConstBlack, ConstColor, ConstWhite, Kind, Piece, Square},
};

const MAX_MOVES: usize = 218;

pub struct MoveEntry {
    r#move: Move,
    score: i16,
}

pub struct Pick {
    moves: [MaybeUninit<MoveEntry>; MAX_MOVES],
    entry: Option<Entry>,
    killer: Option<Move>,
    attacked: Bitboard,
    capture_end: usize,
    quiet_start: usize,
    index: usize,
}

struct PickList<'a> {
    pick: &'a mut Pick,
    tt_move: Move,
    tt_hit: bool,
    killer_hit: bool,
    position: &'a Position,
}

impl MoveList for PickList<'_> {
    fn add_move(&mut self, r#move: Move) {
        if r#move == self.tt_move {
            self.tt_hit = true;

            return;
        }

        if Some(r#move) == self.pick.killer {
            self.killer_hit = true;

            return;
        }

        if self.position.captured_piece(r#move).is_some() {
            self.pick.moves[self.pick.capture_end].write(MoveEntry { r#move, score: 0 });
            self.pick.capture_end += 1;
        } else {
            self.pick.quiet_start -= 1;
            self.pick.moves[self.pick.quiet_start].write(MoveEntry { r#move, score: 0 });
        }
    }
}

impl Pick {
    pub fn new<const QUIET: bool>(
        engine: &Engine,
        generator: &Generator,
        killer: Option<Move>,
    ) -> Self {
        let hash = engine.position().hash();
        let entry = engine.tt().probe(hash);
        let tt_move = entry.map(|entry| entry.r#move()).unwrap_or_else(Move::null);

        let mut pick = Pick {
            moves: [const { MaybeUninit::uninit() }; MAX_MOVES],
            entry,
            killer,
            attacked: generator.attacked(),
            index: 0,
            capture_end: 0,
            quiet_start: MAX_MOVES,
        };

        let mut pick_list = PickList {
            pick: &mut pick,
            tt_move,
            tt_hit: false,
            killer_hit: false,
            position: engine.position(),
        };

        generator.generate_dyn::<QUIET>(&mut pick_list, engine.position());

        if !pick_list.tt_hit {
            pick_list.pick.entry = None;
        }

        if !pick_list.killer_hit {
            pick_list.pick.killer = None;
        }

        pick
    }

    pub fn entry(&self) -> Option<Entry> {
        self.entry
    }

    fn capture_mut(&mut self) -> &mut [MoveEntry] {
        unsafe { mem::transmute(&mut self.moves[..self.capture_end]) }
    }

    fn quiet_mut(&mut self) -> &mut [MoveEntry] {
        unsafe { mem::transmute(&mut self.moves[self.quiet_start..]) }
    }

    fn next_move(&mut self, position: &Position) -> Option<Move> {
        let mut index = self.index;

        if let Some(entry) = self.entry {
            if index == 0 {
                return Some(entry.r#move());
            }

            index -= 1
        }

        if let Some(killer) = self.killer {
            if index == 0 {
                return Some(killer);
            }

            index -= 1;
        }

        let attacked = self.attacked;
        let capture = self.capture_mut();

        if index < capture.len() {
            if index == 0 {
                sort_moves::<true>(attacked, position, capture);
            }

            return Some(capture[index].r#move);
        }

        index -= self.capture_end;

        let quiet = self.quiet_mut();

        if index < quiet.len() {
            if index == 0 {
                sort_moves::<false>(attacked, position, quiet);
            }

            return Some(quiet[index].r#move);
        }

        None
    }

    pub fn next(&mut self, position: &Position) -> Option<(usize, Move)> {
        if let Some(r#move) = self.next_move(position) {
            let index = self.index;

            self.index += 1;

            Some((index, r#move))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entry.is_none() && self.capture_end == 0 && self.quiet_start == MAX_MOVES
    }
}

fn lva<C: ConstColor>(
    global: &GlobalData,
    board: &Board,
    square: Square,
    occupied: Bitboard,
) -> Option<(i16, Bitboard)> {
    let attack = global.attack();
    let magic = global.magic();

    let bb = board.color_bb(C::opponent()) & occupied;

    let pawn_attack = attack.pawn(square, C::color());
    let pawn = bb & board.kind_bb(Kind::Pawn) & pawn_attack;

    if pawn != Bitboard(0) {
        return Some((Kind::Pawn.value(), pawn));
    }

    let knight_attack = attack.knight(square) & bb;
    let knight = board.kind_bb(Kind::Knight) & knight_attack;

    if knight != Bitboard(0) {
        return Some((Kind::Knight.value(), knight));
    }

    let bishop_attack = magic.bishop(square, occupied) & bb;
    let bishop = board.kind_bb(Kind::Bishop) & bishop_attack;

    if bishop != Bitboard(0) {
        return Some((Kind::Bishop.value(), bishop));
    }

    let rook_attack = magic.rook(square, occupied) & bb;
    let rook = board.kind_bb(Kind::Rook) & rook_attack;

    if rook != Bitboard(0) {
        return Some((Kind::Rook.value(), rook));
    }

    let queen_attack = bishop_attack | rook_attack;
    let queen = board.kind_bb(Kind::Queen) & queen_attack;

    if queen != Bitboard(0) {
        return Some((Kind::Queen.value(), queen));
    }

    let king_attack = attack.king(square) & bb;
    let king = board.kind_bb(Kind::King) & king_attack;

    if king != Bitboard(0) {
        return Some((Kind::King.value(), king));
    }

    None
}

fn see<C: ConstColor, const CAPTURE: bool>(
    global: &GlobalData,
    position: &Position,
    kind: Kind,
    r#move: Move,
) -> i16 {
    let mut occupied = position.occupied_bb() ^ Bitboard::from(r#move.from());
    let mut stack = [0; 32];
    let mut depth = 0;
    let mut value = kind.value();
    let mut accum = 0;

    if CAPTURE {
        let (capture, square) = position.captured_piece(r#move).unwrap();

        accum = capture.kind().value();
        occupied &= !Bitboard::from(square);
    }

    while let Some((new_value, bb)) = lva::<C>(global, position, r#move.to(), occupied) {
        occupied ^= bb & -bb;
        stack[depth] = accum;
        depth += 1;
        accum = value - accum;
        value = new_value;

        let Some((new_value, bb)) = lva::<C::Opponent>(global, position, r#move.to(), occupied)
        else {
            break;
        };

        occupied ^= bb & -bb;
        stack[depth] = accum;
        depth += 1;
        accum = value - accum;
        value = new_value;
    }

    for value in stack[..depth].iter().rev() {
        accum = i16::min(*value, -accum);
    }

    accum
}

fn see_dyn<const CAPTURE: bool>(
    global: &GlobalData,
    position: &Position,
    piece: Piece,
    r#move: Move,
) -> i16 {
    match piece.color() {
        Color::White => see::<ConstWhite, CAPTURE>(global, position, piece.kind(), r#move),
        Color::Black => see::<ConstBlack, CAPTURE>(global, position, piece.kind(), r#move),
    }
}

fn sort_moves<const CAPTURE: bool>(
    attacked: Bitboard,
    position: &Position,
    moves: &mut [MoveEntry],
) {
    let global = GlobalData::get();
    let piece_square = global.square();
    let phase = position.phase();

    for entry in &mut *moves {
        let old_piece = position.get(entry.r#move.from()).unwrap();
        let bb = Bitboard::from(entry.r#move.from()) | Bitboard::from(entry.r#move.to());

        let new_piece = entry
            .r#move
            .kind()
            .map(|kind| Piece::new(old_piece.color(), kind))
            .unwrap_or(old_piece);

        let see_score = if bb & attacked != Bitboard(0) {
            see_dyn::<CAPTURE>(global, position, new_piece, entry.r#move)
        } else if CAPTURE {
            position
                .captured_piece(entry.r#move)
                .unwrap()
                .0
                .kind()
                .value()
        } else {
            0
        };

        entry.score = see_score + piece_square.get(old_piece, entry.r#move.to(), phase)
            - piece_square.get(old_piece, entry.r#move.from(), phase)
            + new_piece.kind().value()
            - old_piece.kind().value();
    }

    moves.sort_unstable_by_key(|entry| -entry.score);

    // let piece = engine.position().get(r#move.from()).unwrap();

    // if let Some(capture) = engine.position().get(r#move.to()) {
    //     const VALUE: [i32; 6] = [0, 1, 2, 3, 4, 5];

    //     return *piece.kind().index(&VALUE) + (5 - *capture.kind().index(&VALUE)) * 6;
    // }

    // 1000000
    //     + (piece_square.get(piece, r#move.from(), phase)
    //         - piece_square.get(piece, r#move.to(), phase)) as i32

    // TODO: killer moves, history heuristic, SEE
}
