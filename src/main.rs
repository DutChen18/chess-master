pub mod attack;
pub mod bitboard;
pub mod board;
pub mod engine;
pub mod gen;
pub mod global;
pub mod magic;
pub mod r#move;
pub mod piecesquaretable;
pub mod position;
pub mod search;
pub mod searchlimits;
pub mod shift;
pub mod tt;
pub mod types;
pub mod zobrist;

use r#move::Move;
use position::Position;

use crate::{engine::Engine, types::*};

fn main() {
    let mut engine = Engine::new();

    let mut pos = Position::new();
    eprintln!("{}", pos.evaluate());
    pos.make(Move::new(Square::A2, Square::A4));
    eprintln!("{}", pos.evaluate());
    pos.make(Move::new(Square::B1, Square::C3));
    eprintln!("{}", pos.evaluate());

    engine.uci_run();
}
