use std::sync::LazyLock;
use std::time::Instant;

use crate::attack::AttackTable;
use crate::magic::MagicTable;
use crate::piecesquaretable::PieceSquareTable;
use crate::zobrist::ZobristTable;

pub struct GlobalData {
    zobrist: ZobristTable,
    magic: MagicTable,
    attack: AttackTable,
    piece_square: PieceSquareTable,
}

impl GlobalData {
    pub fn get() -> &'static GlobalData {
        static DATA: LazyLock<GlobalData> = LazyLock::new(GlobalData::new);

        &*DATA
    }

    pub fn new() -> Self {
        fn time<T>(name: &str, func: fn() -> T) -> T {
            let start = Instant::now();
            let value = (func)();

            eprintln!("init {} took {:?}", name, start.elapsed());

            value
        }

        Self {
            zobrist: time("zobrist table", ZobristTable::new),
            magic: time("magic table", MagicTable::new),
            attack: time("attack table", AttackTable::new),
            piece_square: time("piece square table", PieceSquareTable::new),
        }
    }

    pub fn zobrist(&self) -> &ZobristTable {
        &self.zobrist
    }

    pub fn magic(&self) -> &MagicTable {
        &self.magic
    }

    pub fn attack(&self) -> &AttackTable {
        &self.attack
    }

    pub fn piece_square(&self) -> &PieceSquareTable {
        &self.piece_square
    }
}
