use crate::types::{Color, File, Rank, Square};

use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, MulAssign,
    Neg, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use std::mem;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn square(self) -> Option<Square> {
        match self.0.trailing_zeros() {
            square @ 0..64 => Some(unsafe { mem::transmute(square as u8) }),
            _ => None,
        }
    }

    pub fn pop(&mut self) {
        self.0 &= self.0 - 1;
    }

    pub fn r#for(self, color: Color) -> Bitboard {
        match color {
            Color::White => self,
            Color::Black => Self(self.0.swap_bytes()),
        }
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let square = self.square(); 
        self.0 &= self.0.wrapping_sub(1);
        square
    }

    fn count(self) -> usize {
        self.0.count_ones() as usize
    }
}

macro_rules! binary_wrapping_impl {
    ($op:ident, $fn:ident, $op_assign:ident, $fn_assign:ident, $fn_wrapping:ident) => {
        impl $op for Bitboard {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$fn_wrapping(rhs.0))
            }
        }

        impl $op_assign for Bitboard {
            fn $fn_assign(&mut self, rhs: Self) {
                *self = self.$fn(rhs);
            }
        }
    };
}

binary_wrapping_impl!(Add, add, AddAssign, add_assign, wrapping_add);
binary_wrapping_impl!(Sub, sub, SubAssign, sub_assign, wrapping_sub);
binary_wrapping_impl!(Mul, mul, MulAssign, mul_assign, wrapping_mul);

macro_rules! binary_impl {
    ($op:ident, $fn:ident, $op_assign:ident, $fn_assign:ident) => {
        impl $op for Bitboard {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$fn(rhs.0))
            }
        }

        impl $op_assign for Bitboard {
            fn $fn_assign(&mut self, rhs: Self) {
                self.0.$fn_assign(rhs.0);
            }
        }
    };
}

binary_impl!(BitAnd, bitand, BitAndAssign, bitand_assign);
binary_impl!(BitOr, bitor, BitOrAssign, bitor_assign);
binary_impl!(BitXor, bitxor, BitXorAssign, bitxor_assign);

macro_rules! shift_impl {
    ($op:ident, $fn:ident, $op_assign:ident, $fn_assign:ident, $ty:ty) => {
        impl $op<$ty> for Bitboard {
            type Output = Self;

            fn $fn(self, rhs: $ty) -> Self::Output {
                Self(self.0.$fn(rhs))
            }
        }

        impl $op_assign<$ty> for Bitboard {
            fn $fn_assign(&mut self, rhs: $ty) {
                self.0.$fn_assign(rhs);
            }
        }
    };
}

shift_impl!(Shl, shl, ShlAssign, shl_assign, i8);
shift_impl!(Shl, shl, ShlAssign, shl_assign, i16);
shift_impl!(Shl, shl, ShlAssign, shl_assign, i32);
shift_impl!(Shl, shl, ShlAssign, shl_assign, i64);
shift_impl!(Shl, shl, ShlAssign, shl_assign, i128);
shift_impl!(Shl, shl, ShlAssign, shl_assign, isize);
shift_impl!(Shl, shl, ShlAssign, shl_assign, u8);
shift_impl!(Shl, shl, ShlAssign, shl_assign, u16);
shift_impl!(Shl, shl, ShlAssign, shl_assign, u32);
shift_impl!(Shl, shl, ShlAssign, shl_assign, u64);
shift_impl!(Shl, shl, ShlAssign, shl_assign, u128);
shift_impl!(Shl, shl, ShlAssign, shl_assign, usize);

shift_impl!(Shr, shr, ShrAssign, shr_assign, i8);
shift_impl!(Shr, shr, ShrAssign, shr_assign, i16);
shift_impl!(Shr, shr, ShrAssign, shr_assign, i32);
shift_impl!(Shr, shr, ShrAssign, shr_assign, i64);
shift_impl!(Shr, shr, ShrAssign, shr_assign, i128);
shift_impl!(Shr, shr, ShrAssign, shr_assign, isize);
shift_impl!(Shr, shr, ShrAssign, shr_assign, u8);
shift_impl!(Shr, shr, ShrAssign, shr_assign, u16);
shift_impl!(Shr, shr, ShrAssign, shr_assign, u32);
shift_impl!(Shr, shr, ShrAssign, shr_assign, u64);
shift_impl!(Shr, shr, ShrAssign, shr_assign, u128);
shift_impl!(Shr, shr, ShrAssign, shr_assign, usize);

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Neg for Bitboard {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.wrapping_neg())
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Self(1 << square as u8)
    }
}

impl From<File> for Bitboard {
    fn from(file: File) -> Self {
        Self(0x0101010101010101 << file as u8)
    }
}

impl From<Rank> for Bitboard {
    fn from(rank: Rank) -> Self {
        Self(0xFF << (rank as u8 * 8))
    }
}
