use std::mem;

use crate::{
    bitboard::Bitboard,
    types::{Color, ConstColor, Square},
};

pub trait Shift {
    fn shift(&self, bitboard: Bitboard) -> Bitboard;
    fn apply(&self, square: Square) -> Square;
    fn apply_inverse(&self, square: Square) -> Square;
}

trait Ray {
    fn one(&self) -> impl Shift;
    fn two(&self) -> impl Shift;
    fn four(&self) -> impl Shift;
}

pub struct Offset<const FILE: i32, const RANK: i32>;

macro_rules! define_ray {
    ($name:ident, $one:expr, $two:expr, $four:expr) => {
        pub struct $name;

        impl Ray for $name {
            fn one(&self) -> impl Shift {
                $one
            }

            fn two(&self) -> impl Shift {
                $two
            }

            fn four(&self) -> impl Shift {
                $four
            }
        }
    };
}

define_ray!(No, Offset::<0, 1>, Offset::<0, 2>, Offset::<0, 4>);
define_ray!(Ea, Offset::<1, 0>, Offset::<2, 0>, Offset::<4, 0>);
define_ray!(So, Offset::<0, -1>, Offset::<0, -2>, Offset::<0, -4>);
define_ray!(We, Offset::<-1, 0>, Offset::<-2, 0>, Offset::<-4, 0>);
define_ray!(NoEa, Offset::<1, 1>, Offset::<2, 2>, Offset::<4, 4>);
define_ray!(SoEa, Offset::<1, -1>, Offset::<2, -2>, Offset::<4, -4>);
define_ray!(SoWe, Offset::<-1, -1>, Offset::<-2, -2>, Offset::<-4, -4>);
define_ray!(NoWe, Offset::<-1, 1>, Offset::<-2, 2>, Offset::<-4, 4>);

impl<const FILE: i32, const RANK: i32> Offset<FILE, RANK> {
    const fn mask() -> Bitboard {
        if FILE < 0 {
            Bitboard(0x0101010101010101 * (0xFF >> -FILE & 0xFF))
        } else {
            Bitboard(0x0101010101010101 * (0xFF << FILE & 0xFF))
        }
    }
}

impl<const FILE: i32, const RANK: i32> Shift for Offset<FILE, RANK> {
    fn shift(&self, bitboard: Bitboard) -> Bitboard {
        let shift = FILE + RANK * 8;
        let mask = Self::mask();

        if shift < 0 {
            bitboard >> -shift & mask
        } else {
            bitboard << shift & mask
        }
    }

    fn apply(&self, square: Square) -> Square {
        let square = square as i32 + FILE + RANK * 8;

        if square >= 0 && square < 64 {
            unsafe { mem::transmute(square as u8) }
        } else {
            panic!("bad square after shift: {square}");
        }
    }

    fn apply_inverse(&self, square: Square) -> Square {
        let square = square as i32 - FILE - RANK * 8;

        if square >= 0 && square < 64 {
            unsafe { mem::transmute(square as u8) }
        } else {
            panic!("bad square after shift: {square}");
        }
    }
}

fn ray(ray: impl Ray, mut gen: Bitboard, mut pro: Bitboard) -> Bitboard {
    gen |= pro & ray.one().shift(gen);
    pro &= ray.one().shift(pro);
    gen |= pro & ray.two().shift(gen);
    pro &= ray.two().shift(pro);
    gen |= pro & ray.four().shift(gen);
    ray.one().shift(gen)
}

pub fn pawn_attack<C: ConstColor>(bb: Bitboard) -> Bitboard {
    if C::color() == Color::White {
        Offset::<1, 1>.shift(bb) | Offset::<-1, 1>.shift(bb)
    } else {
        Offset::<1, -1>.shift(bb) | Offset::<-1, -1>.shift(bb)
    }
}

pub fn knight_attack(bb: Bitboard) -> Bitboard {
    let h1 = Offset::<1, 0>.shift(bb) | Offset::<-1, 0>.shift(bb);
    let h2 = Offset::<2, 0>.shift(bb) | Offset::<-2, 0>.shift(bb);
    let v2 = Offset::<0, 2>.shift(h1) | Offset::<0, -2>.shift(h1);
    let v1 = Offset::<0, 1>.shift(h2) | Offset::<0, -1>.shift(h2);

    v1 | v2
}

pub fn king_attack(mut bb: Bitboard) -> Bitboard {
    let attacks = Offset::<1, 0>.shift(bb) | Offset::<-1, 0>.shift(bb);
    bb |= attacks;
    attacks | Offset::<0, 1>.shift(bb) | Offset::<0, -1>.shift(bb)
}

pub fn bishop_ray(bb: Bitboard, empty: Bitboard) -> Bitboard {
    let ne = ray(NoEa, bb, empty);
    let se = ray(SoEa, bb, empty);
    let sw = ray(SoWe, bb, empty);
    let nw = ray(NoWe, bb, empty);

    ne | se | sw | nw
}

pub fn rook_ray(bb: Bitboard, empty: Bitboard) -> Bitboard {
    let n = ray(No, bb, empty);
    let e = ray(Ea, bb, empty);
    let s = ray(So, bb, empty);
    let w = ray(We, bb, empty);

    n | e | s | w
}
