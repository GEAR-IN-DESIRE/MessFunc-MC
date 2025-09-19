use crate::pos::{SChunkPos, WChunkPos};
use crate::server::chunk::SChunk;
use crate::server::global::{global, MAX_THREAD_COUNT, MAX_SCHUNK_DECAY_RATE};
use crate::server::ticker::ChunkServerEvent::{RemoveSCEvent, RequestSCEvent, TickEndEvent};
use crate::server::ticker::ChunkServerState::{IsPending, IsTicked, IsTicking};
use crate::server::ChunkServer;
use crate::world::chunk::WChunk;
use crate::{end, start};
use messfunc_rust_lib::{ptr_to_mut, OnetimeSender};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::mem;
use tokio::time::{Instant, Interval};
use twox_hash::XxHash64;

pub struct Ticker {
    /// 所有的 ChunkServer, 在准备 tick_chunk 时会与 pendings 交换, 在 ChunkServer tick 结束时, 返回这里
    pub servers: HashMap<u64, Box<ChunkServer>, BuildHasherDefault<XxHash64>>,
    /// 正在等待让出线程的 ChunkServer
    pub pendings: HashMap<u64, Box<ChunkServer>, BuildHasherDefault<XxHash64>>,
    /// 正在等待 SChunk 的 ChunkServer
    pub waitings: HashMap<u64, Responser, BuildHasherDefault<XxHash64>>,
    /// 用于暂存 ChunkServer 在结束时提交的全局任务, TODO 可以优化为Vec<Vec> 进一步降低开销
    pub tasks: Vec<Box<dyn FnOnce(&mut Ticker) + Send + Sync>>,
    /// 用于tick定时, 他将会自动处理追帧相关的操作
    pub interval: Interval,
    /// 用于获取mspt
    pub tick_start_time: Instant,
    /// 用于控制是否关闭服务器
    pub scheduled_run: bool,
    /// 用于标记 SChunk 已经被申请
    pub schunk_pos_map: HashMap<SChunkPos, u64, BuildHasherDefault<XxHash64>>,
    /// 记录线程数量, 不可超过 MAX_THREAD
    pub thread_count: usize,
    /// 用于构建 ChunkServer
    pub server_builder: u64,
}
impl Ticker {
    pub fn run(mut self) {
        while self.scheduled_run {
            global().runtime.block_on(self.interval.tick());
            self.tick_start_time = Instant::now();
            //
            start(&mut self);
            self.tick_chunk();
            end(&mut self);
            //
            unsafe { ptr_to_mut(global())
                .adjust_record_performance(
                    self.interval.period(),
                    self.tick_start_time.elapsed()
                )
            }
            dbg!(global().mspt);
        }
        // TODO: 在这里实现关闭逻辑
    }
    fn tick_chunk(&mut self) {
        mem::swap(&mut self.pendings, &mut self.servers);
        #[cfg(debug_assertions)]
        println!("chunk server 数量: {}", self.pendings.len());
        for (_, server) in self.pendings.extract_if(|_, _| true) {
            self.thread_count += 1;
            global().runtime.spawn_blocking(|| server.tick());
            if self.thread_count == MAX_THREAD_COUNT { break }
        }
        while self.thread_count != 0 {
            // 此处是目前唯一的可以实现 主线程 与 Chunk线程 并行计算的位置
            // 在没有更多的 ChunkServer 维持线程数量时, 可以开启一些独立任务填充
            match global().runtime.block_on(global().events.wait_recv()) {
                TickEndEvent(server) => self.handle_tick_end(server),
                RequestSCEvent(responser) => self.handle_request_sc(responser),
                RemoveSCEvent(pos) => self.handle_remove_schunk(pos),
            }
        }
    }
    
    fn handle_tick_end(&mut self, mut server: Box<ChunkServer>) {
        while let Some(borrow) = server.borrows.pop() {
            match borrow {
                IsTicking(responser) => self.handle_request_sc(responser),
                IsTicked(server) => self.handle_tick_end(server),
                IsPending(server) => {
                    self.pendings.insert(server.id, server);
                }
            }
        }
        for (_, responser) in self.waitings.extract_if(|_, responser| {
            server.sc_map.contains_key(&responser.req_cell.sc_pos)
        }) {
            responser.response(IsTicked(server));
            return;
        }
        let decay_rate = global().schunk_decay_rate;
        let decay_max = MAX_SCHUNK_DECAY_RATE;
        //
        self.tasks.append(&mut server.global_tasks);
        // 管理合并与拆分以及释放空Server
        if decay_rate < decay_max {
            for i in (0..server.borrows.len()).rev() {
                if let IsTicked(_) = server.borrows[i] {
                    #[cfg(debug_assertions)]
                    println!("server 合并");
                    let mut borrow = server.borrows.remove(i).remove();
                    // TODO: 这里可以免除两次哈希
                    for (pos, sc) in borrow.sc_map.drain() {
                        server.sc_map.insert(pos, sc);
                        self.schunk_pos_map.insert(pos, server.id);
                    }
                    self.thread_count -= 1;
                }
            }
        }
        if decay_rate != 0 && server.sc_map.len() > 1 {
            for (pos, mut sc) in server.sc_map.extract_if(|_, sc| {
                sc.dep_level = sc.dep_level.saturating_sub(decay_rate);
                sc.dep_level == 0
            }) {
                #[cfg(debug_assertions)]
                println!("server拆分");
                sc.dep_level = MAX_SCHUNK_DECAY_RATE;
                let mut new = self.server_builder.build();
                //TODO 这里计算了两次重复的哈希
                new.sc_map.insert(pos, sc);
                self.schunk_pos_map.insert(pos, new.id);
                self.servers.insert(new.id, new);
            }
        }
        if !server.sc_map.is_empty() {
            self.servers.insert(server.id, server);
        }
        self.thread_count -= 1;
        // TODO HashMap内部数据是不连续的, 需要优化空桶遍历
        for (_, server) in self.pendings.extract_if(|_, _| true) {
            global().runtime.spawn_blocking(|| server.tick());
            self.thread_count += 1;
            if self.thread_count == MAX_THREAD_COUNT {
                break;
            }
        }
        // TODO self.thread_count < MAX_THREAD_COUNT 时线程会闲下来了. 可以在此处安排更多任务, 访问可变Global任然是不安全的
        #[cfg(debug_assertions)]
        if self.thread_count < MAX_THREAD_COUNT {

        }
    }
    fn handle_request_sc(&mut self, responser: Responser) {
        let request_pos = responser.req_cell.sc_pos;
        match self.schunk_pos_map.entry(request_pos) {
            Entry::Occupied(o) => {
                let server_id = o.get();
                if let Some(borrow) = self.servers.remove(server_id) {
                    responser.response(IsTicked(borrow));
                    return;
                }
                if let Some(borrow) = self.waitings.remove(server_id) {
                    responser.response(IsTicking(borrow));
                    return;
                }
                if let Some(borrow) = self.pendings.remove(server_id) {
                    responser.response(IsPending(borrow));
                    return;
                }
                self.waitings.insert(responser.req_cell.requester.id, responser);
            }
            Entry::Vacant(v) => {
                let mut borrow = self.server_builder.build();
                borrow.sc_map.insert(request_pos, SChunk::new(request_pos).into());
                v.insert(borrow.id);
                responser.response(IsTicked(borrow));
            }
        }
    }
    fn handle_remove_schunk(&mut self, pos: SChunkPos) {
        for (_, responser) in self.waitings.extract_if(|_, responser| {
            responser.req_cell.sc_pos == pos
        }) {
            let mut server = self.server_builder.build();
            server.sc_map.insert(pos, SChunk::new(pos).into());
            self.schunk_pos_map.insert(pos, server.id);
            responser.response(IsTicked(server));
            return;
        }
        self.schunk_pos_map.remove(&pos).expect("数据混乱");
    }
    pub fn load_wchunk(&mut self, pos: WChunkPos) -> &mut WChunk {
        let sc_pos = pos.to_schunk_pos();
        self.get_chunk_server(sc_pos).sc_map.get_mut(&sc_pos).expect("数据混乱").load_wchunk(pos)
    }

    pub fn remove_wchunk(&mut self, pos: &WChunkPos) -> Option<Box<WChunk>> {
        let sc_pos = pos.to_schunk_pos();
        if let Some(id) = self.schunk_pos_map.remove(&sc_pos) {
            let server = self.servers.get_mut(&id).expect("数据混乱");
            match server.sc_map.entry(sc_pos) {
                Entry::Occupied(mut o) => {
                    let sc = o.get_mut();
                    let wchunk = sc.remove_wchunk(pos);
                    if sc.wchunk_count == 0 {
                        o.remove();
                    }
                    return wchunk
                }
                Entry::Vacant(_) => unreachable!("数据混乱"),
            }
        }
        None
    }

    pub fn get_chunk(&mut self, pos: &WChunkPos) -> Option<&mut Box<WChunk>> {
        let sc_pos = pos.to_schunk_pos();
        self.get_chunk_server(sc_pos).sc_map.get_mut(&sc_pos).expect("数据混乱").get_wchunk(pos)
    }
    fn get_chunk_server(&mut self, sc_pos: SChunkPos) -> &mut ChunkServer {
        match self.schunk_pos_map.entry(sc_pos) {
            Entry::Occupied(o) => {
                self.servers.get_mut(o.get()).expect("数据混乱")
            }
            Entry::Vacant(v) => {
                let mut server = self.server_builder.build();
                v.insert(server.id);
                server.sc_map.insert(sc_pos, SChunk::new(sc_pos).into());
                match self.servers.entry(server.id) {
                    Entry::Occupied(_) => unreachable!("数据混乱"),
                    Entry::Vacant(v) => v.insert(server)
                }
            }
        }
    }
}
trait ServerBuilder {
    fn build(&mut self) -> Box<ChunkServer>;
}
pub struct SChunkRequestCell {
    /// 此处的 server box 是一个 unsafe 副本, 原始 box 此时是被挂起无法使用的
    pub requester: Box<ChunkServer>,
    pub sc_pos: SChunkPos,
}
pub struct SChunkResponseCell {
    // pub requester: Box<ChunkServer>,
    pub response: ChunkServerState,
}
pub struct Responser {
    pub req_cell: SChunkRequestCell,
    pub tx: OnetimeSender<SChunkResponseCell>,
}

pub enum ChunkServerEvent {
    TickEndEvent(Box<ChunkServer>),
    RequestSCEvent(Responser),
    RemoveSCEvent(SChunkPos),
}

pub enum ChunkServerState {
    IsPending(Box<ChunkServer>),
    IsTicking(Responser),
    IsTicked(Box<ChunkServer>),
}
impl ServerBuilder for u64 {
    fn build(&mut self) -> Box<ChunkServer> {
        *self += 1;
        ChunkServer {
            id: *self,
            sc_map: HashMap::with_hasher(BuildHasherDefault::<XxHash64>::new()),
            borrows: Vec::new(),
            global_tasks: Vec::new(),
        }.into()
    }
}
impl Responser {
    pub fn response(self, server: ChunkServerState) {
        mem::forget(self.req_cell.requester);
        self.tx.send(SChunkResponseCell {
            // requester: self.req_cell.requester,
            response: server
        });
    }
}
impl ChunkServerState {
    pub fn get_mut(&mut self) -> &mut ChunkServer {
        match self {
            IsTicking(responser) => &mut responser.req_cell.requester,
            IsTicked(server) => server,
            IsPending(server) => server,
        }
    }
    pub fn get(&self) -> &ChunkServer {
        match self {
            IsTicking(responser) => &responser.req_cell.requester,
            IsTicked(server) => server,
            IsPending(server) => server,
        }
    }

    pub fn remove(self) -> Box<ChunkServer> {
        match self {
            IsTicking(responser) => responser.req_cell.requester,
            IsTicked(server) => server,
            IsPending(server) => server,
        }
    }
}