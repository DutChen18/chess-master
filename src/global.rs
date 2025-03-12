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
    square: PieceSquareTable,
}

impl GlobalData {
    pub fn get() -> &'static GlobalData {
        static DATA: LazyLock<GlobalData> = LazyLock::new(GlobalData::new);

        &*DATA
    }

    pub fn new() -> Self {
        fn time<T>(_name: &str, func: fn() -> T) -> T {
            let _start = Instant::now();
            let value = (func)();

            //eprintln!("init {} took {:?}", name, start.elapsed());

            value
        }

        Self {
            zobrist: time("zobrist table", ZobristTable::new),
            magic: time("magic table", MagicTable::new),
            attack: time("attack table", AttackTable::new),
            square: time("piece square table", PieceSquareTable::new),
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

    pub fn square(&self) -> &PieceSquareTable {
        &self.square
    }
}
