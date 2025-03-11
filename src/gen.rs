use std::mem::{self, MaybeUninit};

use crate::bitboard::Bitboard;
use crate::global::GlobalData;
use crate::position::Position;
use crate::r#move::Move;
use crate::shift::{self, Shift};
use crate::types::{Color, ConstBlack, ConstColor, ConstWhite, Kind, Square};

pub trait MoveList {
    fn add_move(&mut self, r#move: Move);

    fn add<const PROMOTION: bool>(&mut self, from: Square, to: Square) {
        if PROMOTION {
            self.add_move(Move::new_promotion(from, to, Kind::Knight));
            self.add_move(Move::new_promotion(from, to, Kind::Bishop));
            self.add_move(Move::new_promotion(from, to, Kind::Rook));
            self.add_move(Move::new_promotion(from, to, Kind::Queen));
        } else {
            self.add_move(Move::new(from, to));
        }
    }

    fn add_bb<const PROMOTION: bool>(&mut self, from: Square, to: Bitboard) {
        for to in to {
            self.add::<PROMOTION>(from, to);
        }
    }

    fn add_shift<const PROMOTION: bool>(&mut self, shift: &impl Shift, to: Bitboard) {
        for to in to {
            self.add::<PROMOTION>(shift.apply_inverse(to), to);
        }
    }

    fn set_check(&mut self) {}
}

pub struct MoveVec {
    moves: [MaybeUninit<Move>; 218],
    count: usize,
    check: bool,
}

fn generate_pawn_bb<C: ConstColor>(list: &mut impl MoveList, from: Square, to: Bitboard) {
    let rank_8 = Bitboard(0xFF00000000000000).r#for(C::color());

    list.add_bb::<false>(from, to & !rank_8);
    list.add_bb::<true>(from, to & rank_8);
}

fn generate_pawn_shift<C: ConstColor>(list: &mut impl MoveList, shift: &impl Shift, to: Bitboard) {
    let rank_8 = Bitboard(0xFF00000000000000).r#for(C::color());

    list.add_shift::<false>(shift, to & !rank_8);
    list.add_shift::<true>(shift, to & rank_8);
}

pub fn generate<C: ConstColor, const QUIET: bool>(list: &mut impl MoveList, position: &Position) {
    let global = GlobalData::get();
    let magic = global.magic();
    let attack = global.attack();

    let own_king = position.king_square(C::color());
    let opp_king = position.king_square(C::opponent());
    let own = position.color_bb(C::color());
    let opp = position.color_bb(C::opponent());
    let occupied = own | opp;

    let own_pawn = position.color_kind_bb(C::color(), Kind::Pawn);
    let own_knight = position.color_kind_bb(C::color(), Kind::Knight);
    let own_bishop = position.bishop_queen_bb(C::color());
    let own_rook = position.rook_queen_bb(C::color());
    let opp_pawn = position.color_kind_bb(C::opponent(), Kind::Pawn);
    let opp_knight = position.color_kind_bb(C::opponent(), Kind::Knight);
    let opp_bishop = position.bishop_queen_bb(C::opponent());
    let opp_rook = position.rook_queen_bb(C::opponent());

    let rank_3 = Bitboard(0xFF0000).r#for(C::color());

    let mut checkers = Bitboard(0);
    let mut pinners = Bitboard(0);
    let mut pinned = Bitboard(0);
    let mut target = !own;
    let mut attacked = Bitboard(0);

    let bb = magic.bishop(own_king, occupied);
    checkers |= bb & opp_bishop;
    pinners |= (magic.bishop(own_king, occupied ^ (bb & own)) ^ bb) & opp_bishop;
    let bb = magic.rook(own_king, occupied);
    checkers |= bb & opp_rook;
    pinners |= (magic.rook(own_king, occupied ^ (bb & own)) ^ bb) & opp_rook;

    for pinner in pinners {
        pinned |= attack.between(own_king, pinner) & own;
    }

    checkers |= attack.pawn(own_king, C::color()) & opp_pawn;
    checkers |= attack.knight(own_king) & opp_knight;

    if let Some(checker) = checkers.square() {
        target &= checkers;
        target |= attack.between(own_king, checker);
    }

    if !QUIET {
        target &= opp;
    }

    attacked |= shift::pawn_attack::<C::Opponent>(opp_pawn);
    attacked |= shift::knight_attack(opp_knight);
    attacked |= attack.king(opp_king);

    let king_bb = Bitboard::from(own_king);

    for bishop in opp_bishop {
        attacked |= magic.bishop(bishop, occupied & !king_bb);
    }

    for rook in opp_rook {
        attacked |= magic.rook(rook, occupied & !king_bb);
    }

    // Not in check
    if checkers == Bitboard(0) {
        // Castling moves
        let castling_rights = position.castling_rights();
        let short_mask = Bitboard(0x70).r#for(C::color());
        let short_path = Bitboard(0x60).r#for(C::color());
        let long_mask = Bitboard(0x1C).r#for(C::color());
        let long_path = Bitboard(0x0E).r#for(C::color());

        if QUIET {
            if castling_rights.has_short(C::color())
                && short_mask & attacked == Bitboard(0)
                && short_path & occupied == Bitboard(0)
            {
                list.add::<false>(Square::E1.r#for(C::color()), Square::G1.r#for(C::color()));
            }

            if castling_rights.has_long(C::color())
                && long_mask & attacked == Bitboard(0)
                && long_path & occupied == Bitboard(0)
            {
                list.add::<false>(Square::E1.r#for(C::color()), Square::C1.r#for(C::color()));
            }
        }

        // Pinned pawn moves
        for pawn in own_pawn & pinned {
            let up = C::up().shift(pawn.into()) & !occupied;
            let up_up = C::up().shift(up & rank_3) & !occupied;
            let up_side = attack.pawn(pawn, C::color()) & opp;

            generate_pawn_bb::<C>(
                list,
                pawn,
                (up | up_up | up_side) & target & attack.line(own_king, pawn),
            );
        }

        // Pinned bishop/queen moves
        for bishop in own_bishop & pinned {
            list.add_bb::<false>(
                bishop,
                magic.bishop(bishop, occupied) & target & attack.line(own_king, bishop),
            );
        }

        // Pinned rook/queen moves
        for rook in own_rook & pinned {
            list.add_bb::<false>(
                rook,
                magic.rook(rook, occupied) & target & attack.line(own_king, rook),
            );
        }
    } else {
        list.set_check();
    }

    // King moves
    let mut bb = attack.king(own_king) & !attacked & !own;

    if !QUIET {
        bb &= opp;
    }

    list.add_bb::<false>(own_king, bb);

    if checkers & (checkers - Bitboard(1)) != Bitboard(0) {
        // Double check, we are done
        return;
    }

    // Pawn moves
    let bb = own_pawn & !pinned;
    let up = C::up().shift(bb) & !occupied;
    let up_up = C::up().shift(up & rank_3) & !occupied;
    let up_left = C::up_left().shift(bb) & opp;
    let up_right = C::up_right().shift(bb) & opp;

    generate_pawn_shift::<C>(list, &C::up(), up & target);
    generate_pawn_shift::<C>(list, &C::up_up(), up_up & target);
    generate_pawn_shift::<C>(list, &C::up_left(), up_left & target);
    generate_pawn_shift::<C>(list, &C::up_right(), up_right & target);

    // En passant moves
    if let Some(to) = position.en_passant() {
        for from in own_pawn & attack.pawn(to, C::opponent()) {
            let new_occupied = occupied
                ^ Bitboard::from(from)
                ^ Bitboard::from(to)
                ^ Bitboard::from(C::up().apply_inverse(to));

            let bishop_attack = magic.bishop(own_king, new_occupied) & opp_bishop;
            let rook_attack = magic.rook(own_king, new_occupied) & opp_rook;

            if bishop_attack | rook_attack == Bitboard(0) {
                list.add::<false>(from, to);
            }
        }
    }

    // Knight moves
    for knight in own_knight & !pinned {
        list.add_bb::<false>(knight, attack.knight(knight) & target);
    }

    // Bishop moves
    for bishop in own_bishop & !pinned {
        list.add_bb::<false>(bishop, magic.bishop(bishop, occupied) & target);
    }

    // Rook moves
    for rook in own_rook & !pinned {
        list.add_bb::<false>(rook, magic.rook(rook, occupied) & target);
    }
}

pub fn generate_dyn<const QUIET: bool>(list: &mut impl MoveList, position: &Position) {
    match position.turn() {
        Color::White => generate::<ConstWhite, QUIET>(list, position),
        Color::Black => generate::<ConstBlack, QUIET>(list, position),
    }
}

impl MoveVec {
    pub fn new() -> Self {
        Self {
            moves: [MaybeUninit::uninit(); 218],
            count: 0,
            check: false,
        }
    }

    pub fn moves(&self) -> &[Move] {
        unsafe { mem::transmute(&self.moves[..self.count]) }
    }

    pub fn moves_mut(&mut self) -> &mut [Move] {
        unsafe { mem::transmute(&mut self.moves[..self.count]) }
    }

    pub fn check(&self) -> bool {
        self.check
    }
}

impl Drop for MoveVec {
    fn drop(&mut self) {
        for r#move in &mut self.moves[..self.count] {
            unsafe { r#move.assume_init_drop() };
        }
    }
}

impl MoveList for MoveVec {
    fn add_move(&mut self, r#move: Move) {
        self.moves[self.count].write(r#move);
        self.count += 1;
    }

    fn set_check(&mut self) {
        self.check = true;
    }
}

impl MoveList for Vec<Move> {
    fn add_move(&mut self, r#move: Move) {
        self.push(r#move);
    }
}

impl MoveList for usize {
    fn add_move(&mut self, _: Move) {
        *self += 1;
    }

    fn add<const PROMOTION: bool>(&mut self, _: Square, _: Square) {
        if PROMOTION {
            *self += 4;
        } else {
            *self += 1;
        }
    }

    fn add_bb<const PROMOTION: bool>(&mut self, _: Square, to: Bitboard) {
        if PROMOTION {
            *self += to.count() * 4;
        } else {
            *self += to.count();
        }
    }

    fn add_shift<const PROMOTION: bool>(&mut self, _: &impl Shift, to: Bitboard) {
        if PROMOTION {
            *self += to.count() * 4;
        } else {
            *self += to.count();
        }
    }
}
