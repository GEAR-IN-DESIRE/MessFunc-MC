use crate::entity::EntityTrait;
use crate::pos::WChunkPos;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use twox_hash::XxHash64;
use uuid::Uuid;

pub struct WChunk {
    pub pos: WChunkPos,
    pub entities: HashMap<Uuid, Box<dyn EntityTrait>, BuildHasherDefault<XxHash64>>,
}

impl WChunk {
    pub fn load(pos: WChunkPos) -> Self {
        WChunk {
            pos,
            entities: HashMap::with_hasher(BuildHasherDefault::<XxHash64>::new()),
        }
    }
}