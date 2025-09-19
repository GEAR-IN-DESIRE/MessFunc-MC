use crate::server::global::{SCHUNK_SIZE, WCHUNK_SIZE};
use messfunc_rust_lib::AtomicBox;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct EntityPos {
    pub world: Uuid,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct BlockPos {
    pub world: Uuid,
    pub x: i64,
    pub z: i64,
}
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct WChunkPos {
    pub world: Uuid,
    pub x: i64,
    pub z: i64,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct SChunkPos {
    pub world: Uuid,
    pub x: i64,
    pub z: i64,
}
impl EntityPos {
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos {
            world: self.world,
            x: self.x.floor() as i64,
            z: self.z.floor() as i64,
        }
    }
    pub fn to_wchunk_pos(&self) -> WChunkPos {
        let wchunk_size = WCHUNK_SIZE as i64;
        WChunkPos {
            world: self.world,
            x: (self.x.floor() as i64).div_euclid(wchunk_size),
            z: (self.z.floor() as i64).div_euclid(wchunk_size),
        }
    }
    pub fn to_schunk_pos(&self) -> SChunkPos {
        let sc_size = (SCHUNK_SIZE * WCHUNK_SIZE) as i64;
        SChunkPos {
            world: self.world,
            x: (self.x.floor() as i64).div_euclid(sc_size),
            z: (self.z.floor() as i64).div_euclid(sc_size),
        }
    }
}
impl BlockPos {
    pub fn to_wchunk_pos(&self) -> WChunkPos {
        let wchunk_size = WCHUNK_SIZE as i64;
        WChunkPos {
            world: self.world,
            x: self.x.div_euclid(wchunk_size),
            z: self.z.div_euclid(wchunk_size),
        }
    }
    pub fn to_schunk_pos(&self) -> SChunkPos {
        let i = (SCHUNK_SIZE * WCHUNK_SIZE) as i64;
        SChunkPos {
            world: self.world,
            x: self.x.div_euclid(i),
            z: self.z.div_euclid(i),
        }
    }
}

impl WChunkPos {
    pub fn to_schunk_pos(&self) -> SChunkPos {
        let sc_size = SCHUNK_SIZE as i64;
        SChunkPos {
            world: self.world,
            x: self.x.div_euclid(sc_size),
            z: self.z.div_euclid(sc_size),
        }
    }
}
pub type SyncEntityPos = AtomicBox<EntityPos>;