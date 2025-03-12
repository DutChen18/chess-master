use std::fmt;
use std::mem;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::shift::Offset;
use crate::shift::Shift;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: CastlingRights = CastlingRights(0b0000);
    pub const ALL: CastlingRights = CastlingRights(0b1111);
    pub const WHITE: CastlingRights = CastlingRights(0b0011);
    pub const BLACK: CastlingRights = CastlingRights(0b1100);
    pub const SHORT: CastlingRights = CastlingRights(0b0101);
    pub const LONG: CastlingRights = CastlingRights(0b1010);
    pub const WHITE_SHORT: CastlingRights = CastlingRights(0b0001);
    pub const WHITE_LONG: CastlingRights = CastlingRights(0b0010);
    pub const BLACK_SHORT: CastlingRights = CastlingRights(0b0100);
    pub const BLACK_LONG: CastlingRights = CastlingRights(0b1000);

    pub fn has(self, other: Self) -> bool {
        self & other != Self::NONE
    }

    pub fn has_short(self, color: Color) -> bool {
        self.has(match color {
            Color::White => CastlingRights::WHITE_SHORT,
            Color::Black => CastlingRights::BLACK_SHORT,
        })
    }

    pub fn has_long(self, color: Color) -> bool {
        self.has(match color {
            Color::White => CastlingRights::WHITE_LONG,
            Color::Black => CastlingRights::BLACK_LONG,
        })
    }

    pub fn from_str(s: &str) -> Self {
        let mut castling_rights = Self::NONE;

        for ch in s.chars() {
            match ch {
                'K' => castling_rights |= Self::WHITE_SHORT,
                'Q' => castling_rights |= Self::WHITE_LONG,
                'k' => castling_rights |= Self::BLACK_SHORT,
                'q' => castling_rights |= Self::BLACK_LONG,
                _ => {}
            }
        }

        castling_rights
    }

    pub fn index<T>(self, array: &[T; 16]) -> &T {
        unsafe { &array.get_unchecked(self.0 as usize) }
    }
}

macro_rules! binary_impl {
    ($op:ident, $fn:ident, $op_assign:ident, $fn_assign:ident) => {
        impl $op for CastlingRights {
            type Output = Self;

            fn $fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$fn(rhs.0))
            }
        }

        impl $op_assign for CastlingRights {
            fn $fn_assign(&mut self, rhs: Self) {
                self.0.$fn_assign(rhs.0);
            }
        }
    };
}

binary_impl!(BitAnd, bitand, BitAndAssign, bitand_assign);
binary_impl!(BitOr, bitor, BitOrAssign, bitor_assign);
binary_impl!(BitXor, bitxor, BitXorAssign, bitxor_assign);

impl Not for CastlingRights {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

macro_rules! count {
    () => { 0 };
    ($first:tt $($rest:tt)*) => { 1 + count!($($rest)*) };
}

macro_rules! define_enum {
    ($(
        $(#[$meta:meta])*
        $vis:vis enum $ident:ident {
            $($variant:ident,)+
        }
    )*) => {
        $(
            $(#[$meta])*
            #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
            $vis enum $ident {
                $($variant,)*
            }

            impl $ident {
                pub const COUNT: usize = count!($($variant)*);

                pub fn index<T>(self, array: &[T; Self::COUNT]) -> &T {
                    unsafe { array.get_unchecked(self as usize) }
                }

                pub fn index_mut<T>(self, array: &mut [T; Self::COUNT]) -> &mut T {
                    unsafe { array.get_unchecked_mut(self as usize) }
                }

                pub fn iter() -> impl Iterator<Item = Self> + DoubleEndedIterator + ExactSizeIterator {
                    (0..Self::COUNT).map(|repr| unsafe { mem::transmute(repr as u8) })
                }
            }
        )*
    }
}

define_enum! {
    pub enum Color {
        White,
        Black,
    }

    pub enum Kind {
        Pawn,
        Knight,
        Bishop,
        Rook,
        Queen,
        King,
    }

    pub enum Piece {
        WhitePawn,
        BlackPawn,
        WhiteKnight,
        BlackKnight,
        WhiteBishop,
        BlackBishop,
        WhiteRook,
        BlackRook,
        WhiteQueen,
        BlackQueen,
        WhiteKing,
        BlackKing,
    }

    pub enum File {
        A, B, C, D, E, F, G, H,
    }

    pub enum Rank {
        _1, _2, _3, _4, _5, _6, _7, _8,
    }

    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8,
    }

    pub enum Phase {
        Opening,
        Middle,
        Endgame,
    }
}

pub trait ConstColor {
    type Opponent: ConstColor;

    fn color() -> Color;

    fn opponent() -> Color {
        !Self::color()
    }

    fn up() -> impl Shift;
    fn up_up() -> impl Shift;
    fn up_left() -> impl Shift;
    fn up_right() -> impl Shift;
    fn down() -> impl Shift;
}

pub struct ConstWhite;
pub struct ConstBlack;

impl ConstColor for ConstWhite {
    type Opponent = ConstBlack;

    fn color() -> Color {
        Color::White
    }

    fn up() -> impl Shift {
        Offset::<0, 1>
    }

    fn up_up() -> impl Shift {
        Offset::<0, 2>
    }

    fn up_left() -> impl Shift {
        Offset::<-1, 1>
    }

    fn up_right() -> impl Shift {
        Offset::<1, 1>
    }

    fn down() -> impl Shift {
        Offset::<0, -1>
    }
}

impl ConstColor for ConstBlack {
    type Opponent = ConstWhite;

    fn color() -> Color {
        Color::Black
    }

    fn up() -> impl Shift {
        Offset::<0, -1>
    }

    fn up_up() -> impl Shift {
        Offset::<0, -2>
    }

    fn up_left() -> impl Shift {
        Offset::<-1, -1>
    }

    fn up_right() -> impl Shift {
        Offset::<1, -1>
    }

    fn down() -> impl Shift {
        Offset::<0, 1>
    }
}

impl Color {
    pub fn from_char(ch: char) -> Self {
        match ch {
            'w' => Self::White,
            'b' => Self::Black,
            _ => panic!("bad color {ch}"),
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_char(s.chars().nth(0).unwrap())
    }

    pub fn sign(self) -> i16 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }
}

impl Kind {
    pub fn from_char(ch: char) -> Self {
        match ch {
            'p' => Self::Pawn,
            'n' => Self::Knight,
            'b' => Self::Bishop,
            'r' => Self::Rook,
            'q' => Self::Queen,
            'k' => Self::King,
            _ => panic!("bad piece kind {ch}"),
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_char(s.chars().nth(0).unwrap())
    }

    pub fn to_char(self) -> char {
        match self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        }
    }

    pub const fn value(self) -> i16 {
        match self {
            Self::Pawn => 100,
            Self::Knight | Self::Bishop => 300,
            Self::Rook => 500,
            Self::Queen => 900,
            Self::King => 10000,
        }
    }
}

impl Piece {
    pub const fn new(color: Color, kind: Kind) -> Self {
        unsafe { mem::transmute(color as u8 | (kind as u8) << 1) }
    }

    pub const fn color(self) -> Color {
        unsafe { mem::transmute(self as u8 & 1) }
    }

    pub const fn kind(self) -> Kind {
        unsafe { mem::transmute(self as u8 >> 1) }
    }

    pub fn from_char(ch: char) -> Self {
        Self::new(
            if ch.is_ascii_uppercase() {
                Color::White
            } else {
                Color::Black
            },
            Kind::from_char(ch.to_ascii_lowercase()),
        )
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_char(s.chars().nth(0).unwrap())
    }

    pub fn to_char(self) -> char {
        let c = self.kind().to_char();

        match self.color() {
            Color::White => c.to_uppercase().next().unwrap(),
            Color::Black => c,
        }
    }
}

impl File {
    pub fn from_char(ch: char) -> Self {
        match ch {
            'a' => Self::A,
            'b' => Self::B,
            'c' => Self::C,
            'd' => Self::D,
            'e' => Self::E,
            'f' => Self::F,
            'g' => Self::G,
            'h' => Self::H,
            _ => panic!("bad file {ch}"),
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_char(s.chars().nth(0).unwrap())
    }

    pub fn to_char(self) -> char {
        match self {
            Self::A => 'a',
            Self::B => 'b',
            Self::C => 'c',
            Self::D => 'd',
            Self::E => 'e',
            Self::F => 'f',
            Self::G => 'g',
            Self::H => 'h',
        }
    }
}

impl Rank {
    pub fn from_char(ch: char) -> Self {
        match ch {
            '1' => Self::_1,
            '2' => Self::_2,
            '3' => Self::_3,
            '4' => Self::_4,
            '5' => Self::_5,
            '6' => Self::_6,
            '7' => Self::_7,
            '8' => Self::_8,
            _ => panic!("bad file {ch}"),
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_char(s.chars().nth(0).unwrap())
    }

    pub fn to_char(self) -> char {
        match self {
            Self::_1 => '1',
            Self::_2 => '2',
            Self::_3 => '3',
            Self::_4 => '4',
            Self::_5 => '5',
            Self::_6 => '6',
            Self::_7 => '7',
            Self::_8 => '8',
        }
    }

    pub const fn r#for(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => unsafe { mem::transmute(self as u8 ^ 7) },
        }
    }
}

impl Square {
    pub const fn new(file: File, rank: Rank) -> Self {
        unsafe { mem::transmute(file as u8 | (rank as u8) << 3) }
    }

    pub const fn file(self) -> File {
        unsafe { mem::transmute(self as u8 & 7) }
    }

    pub const fn rank(self) -> Rank {
        unsafe { mem::transmute(self as u8 >> 3) }
    }

    pub fn from_str(s: &str) -> Self {
        let file = File::from_str(&s[0..1]);
        let rank = Rank::from_str(&s[1..2]);

        Self::new(file, rank)
    }

    pub const fn r#for(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => unsafe { mem::transmute(self as u8 ^ 56) },
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.file().to_char(), self.rank().to_char())
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has(Self::WHITE_SHORT) {
            write!(f, "K")?;
        }
        if self.has(Self::WHITE_LONG) {
            write!(f, "Q")?;
        }
        if self.has(Self::BLACK_SHORT) {
            write!(f, "k")?;
        }
        if self.has(Self::BLACK_LONG) {
            write!(f, "q")?;
        }
        if *self == Self::NONE {
            write!(f, "-")?;
        }

        Ok(())
    }
}
