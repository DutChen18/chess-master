use engine::Engine;
use position::Position;

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
pub mod book;
pub mod pick;
pub mod options;

fn main() {
    let mut engine = Engine::new();
    let p = Position::new();

    p.evaluate();

    engine.uci_run();
}
