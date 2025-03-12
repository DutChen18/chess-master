use engine::Engine;

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

fn main() {
    let mut engine = Engine::new();

    engine.uci_run();
}
