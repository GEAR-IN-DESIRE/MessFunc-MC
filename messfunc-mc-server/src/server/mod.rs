use crate::entity::player::Player;
use crate::pos::{SChunkPos, WChunkPos};
use crate::server::chunk::SChunk;
use crate::server::global::global;
use crate::server::ticker::ChunkServerEvent::{RemoveSCEvent, TickEndEvent};
use crate::server::ticker::{ChunkServerEvent, ChunkServerState, Responser, SChunkRequestCell, Ticker};
use crate::tick_chunk;
use crate::world::chunk::WChunk;
use messfunc_rust_lib::OnetimeChannel;
use std::any::Any;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use twox_hash::XxHash64;
use uuid::Uuid;

pub mod global;
pub mod ticker;
pub mod chunk;

pub struct ChunkServer {
    pub id: u64,
    pub sc_map: HashMap<SChunkPos, Box<SChunk>, BuildHasherDefault<XxHash64>>,
    pub borrows: Vec<ChunkServerState>,
    pub global_tasks: Vec<Box<dyn FnOnce(&mut Ticker) + Send + Sync>>,
}
impl ChunkServer {
    pub fn tick(mut self: Box<Self>)  {
        tick_chunk(&mut self);
        global().runtime.spawn(global().events.send(TickEndEvent(self)));
    }

    fn request_schunk(&mut self, pos: SChunkPos) -> &mut SChunk {
        // TODO 如果 get_load_size() 过大那么将被调度到全局锁中访问
        match self.sc_map.get_mut(&pos) {
            None => unsafe {
                // 请求返回的Cell内将会返回自己以及带又请求的sc_key的另一个Server
                let (tx, rx) = OnetimeChannel::new().split();
                let req_cell = SChunkRequestCell {
                    requester: Box::from_raw(self as *mut ChunkServer),
                    sc_pos: pos,
                };
                let responser = Responser {
                    req_cell,
                    tx,
                };
                global().runtime.spawn(global().events.send(ChunkServerEvent::RequestSCEvent(responser)));
                let response_cell = global().runtime.block_on(rx.wait_recv());
                let index = self.borrows.len();
                self.borrows.push(response_cell.response);
                // mem::forget(response_cell.requester);
                self.borrows[index].get_mut().sc_map.get_mut(&pos).expect("数据混乱! 请求到的Server中不包含请求的SChunk")
            },
            Some(sc) => sc,
        }
    }
    pub fn load_wchunk(&mut self, pos: WChunkPos) -> &mut WChunk {
        self.request_schunk(pos.to_schunk_pos()).load_wchunk(pos)
    }

    pub fn remove_wchunk(&mut self, pos: &WChunkPos) -> Option<Box<WChunk>> {
        let sc_pos = pos.to_schunk_pos();
        let sc = self.request_schunk(sc_pos);
        let wchunk = sc.remove_wchunk(pos);
        if sc.wchunk_count == 0 {
            self.sc_map.remove(&sc_pos);
            global().runtime.spawn(global().events.send(RemoveSCEvent(sc_pos)));
        }
        wchunk
    }

    pub fn get_wchunk(&mut self, pos: &WChunkPos) -> Option<&mut Box<WChunk>> {
        self.request_schunk(pos.to_schunk_pos()).get_wchunk(pos)
    }
    pub fn get_player(&mut self, uuid: &Uuid) -> Option<&mut Player> {
        let sync_pos = global().players.get(uuid)?;
        let entity = loop {
            let pos = sync_pos.load().to_wchunk_pos();
            let wchunk = self.load_wchunk(pos);
            if let Some(entity) = wchunk.entities.get_mut(&uuid) {
                break entity;
            }
        };
        (entity as &mut dyn Any).downcast_mut::<Player>()
    }

    pub fn get_load_size(&self) -> usize {
        let mut load_size = self.sc_map.len();
        for cs_state in self.borrows.iter() {
            load_size += cs_state.get().get_load_size();
        }
        load_size
    }
    
}