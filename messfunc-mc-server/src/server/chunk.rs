use crate::pos::{WChunkPos, SChunkPos};
use crate::server::global::{MAX_SCHUNK_DECAY_RATE, SCHUNK_SIZE};
use crate::world::chunk::WChunk;
use std::array;

pub struct SChunk {
    pub wchunk_count: u16,
    pub dep_level: u8,
    pub pos: SChunkPos,
    pub wchunks: [[Option<Box<WChunk>>; SCHUNK_SIZE]; SCHUNK_SIZE],
}

impl SChunk {
    pub fn new(pos: SChunkPos) -> Self {
        Self {
            wchunk_count: 0,
            dep_level: MAX_SCHUNK_DECAY_RATE,
            pos,
            wchunks: array::from_fn(|_| array::from_fn(|_| None)),
        }
    }
    pub fn load_wchunk(&mut self, pos: WChunkPos) -> &mut WChunk {
        let (x, z) = self.get_index(&pos);
        match &mut self.wchunks[x][z] {
            Some(chunk) => chunk,
            slot @ _ => {
                self.wchunk_count += 1;
                slot.insert(WChunk::load(pos).into())
            },
        }
    }
    pub fn remove_wchunk(&mut self, pos: &WChunkPos) -> Option<Box<WChunk>> {
        let (x, z)  = self.get_index(pos);
        let wchunk = self.wchunks[x][z].take();
        if wchunk.is_some() {
            self.wchunk_count -= 1;
        }
        wchunk
    }
    pub fn get_wchunk(&mut self, pos: &WChunkPos) -> Option<&mut Box<WChunk>> {
        let (x, z)  = self.get_index(pos);
        self.wchunks[x][z].as_mut()
    }
    fn get_index(&mut self, pos: &WChunkPos) -> (usize, usize) {
        self.dep_level = MAX_SCHUNK_DECAY_RATE;
        let sc_size = SCHUNK_SIZE as i64;
        let x = ((pos.x % sc_size) + sc_size) % sc_size;
        let z = ((pos.z % sc_size) + sc_size) % sc_size;
        debug_assert!(pos.world == self.pos.world && x < sc_size && z < sc_size,
                      "坐标超出范围: world={:?} x={}, z={}, 有效范围: 0~{}", self.pos.world, x, z, sc_size-1);
        (x as usize, z as usize)
    }
}