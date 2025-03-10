pub mod bitboard;
pub mod board;
pub mod engine;
pub mod magic;
pub mod r#move;
pub mod position;
pub mod searchlimits;
pub mod shift;
pub mod types;
pub mod uci;
pub mod zobrist;

use crate::engine::Engine;

fn main() {
    let mut engine = Engine::new();

    engine.run();
}
