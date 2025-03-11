use crate::types::{Kind, Square};
use std::fmt;
use std::mem;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Move(u16);

impl Move {
    pub fn null() -> Self {
        Self::new(Square::A1, Square::A1)
    }

    pub fn new(from: Square, to: Square) -> Self {
        Self(from as u16 | (to as u16) << 6 | 24576)
    }

    pub fn new_promotion(from: Square, to: Square, kind: Kind) -> Self {
        Self(from as u16 | (to as u16) << 6 | (kind as u16) << 12)
    }

    pub fn from(self) -> Square {
        unsafe { mem::transmute((self.0 & 63) as u8) }
    }

    pub fn to(self) -> Square {
        unsafe { mem::transmute((self.0 >> 6 & 63) as u8) }
    }

    pub fn kind(self) -> Option<Kind> {
        match self.0 >> 12 {
            kind @ 0..6 => Some(unsafe { mem::transmute(kind as u8) }),
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let from = Square::from_str(&s[0..2]);
        let to = Square::from_str(&s[2..4]);

        if s.len() > 4 {
            Self::new_promotion(from, to, Kind::from_str(&s[4..5]))
        } else {
            Self::new(from, to)
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;

        if let Some(kind) = self.kind() {
            write!(f, "{}", kind)?;
        }

        Ok(())
    }
}
