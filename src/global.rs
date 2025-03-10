use std::sync::LazyLock;

use crate::attack::AttackTable;
use crate::magic::MagicTable;
use crate::zobrist::ZobristTable;

pub struct GlobalData {
    zobrist: ZobristTable,
    magic: MagicTable,
    attack: AttackTable,
}

impl GlobalData {
    pub fn get() -> &'static GlobalData {
        static DATA: LazyLock<GlobalData> = LazyLock::new(GlobalData::new);

        &*DATA
    }

    pub fn new() -> Self {
        Self {
            zobrist: ZobristTable::new(),
            magic: MagicTable::new(),
            attack: AttackTable::new(),
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
}
