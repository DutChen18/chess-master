use crate::bitboard::Bitboard;

trait Shift {
    fn shift(&self, bitboard: Bitboard) -> Bitboard;
}

trait Ray {
    fn one(&self) -> impl Shift;
    fn two(&self) -> impl Shift;
    fn four(&self) -> impl Shift;
}

struct Offset<const FILE: i32, const RANK: i32>;

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
            Bitboard(0x0101010101010101 * (0xFF << -FILE & 0xFF))
        } else {
            Bitboard(0x0101010101010101 * (0xFF >> FILE & 0xFF))
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
}

fn ray<R: Ray>(ray: R, mut gen: Bitboard, mut pro: Bitboard) -> Bitboard {
    gen |= pro & ray.one().shift(gen);
    pro &= ray.one().shift(pro);
    gen |= pro & ray.two().shift(gen);
    pro &= ray.two().shift(pro);
    gen |= pro & ray.four().shift(gen);
    ray.one().shift(gen)
}

pub fn bishop_ray(bitboard: Bitboard, empty: Bitboard) -> Bitboard {
    let ne = ray(NoEa, bitboard, empty);
    let se = ray(SoEa, bitboard, empty);
    let sw = ray(SoWe, bitboard, empty);
    let nw = ray(NoWe, bitboard, empty);

    ne | se | sw | nw
}

pub fn rook_ray(bitboard: Bitboard, empty: Bitboard) -> Bitboard {
    let n = ray(No, bitboard, empty);
    let e = ray(Ea, bitboard, empty);
    let s = ray(So, bitboard, empty);
    let w = ray(We, bitboard, empty);

    n | e | s | w
}
