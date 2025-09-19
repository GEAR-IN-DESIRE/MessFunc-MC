use crate::pos::{SyncEntityPos, WChunkPos};
use crate::server::chunk::SChunk;
use crate::server::ticker::ChunkServerEvent;
use crate::world::world::World;
use dashmap::DashMap;
use messfunc_rust_lib::Channel;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::ptr::addr_of;
use std::task::Poll::Pending;
use std::time::Duration;
use sysinfo::System;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use twox_hash::XxHash64;
use uuid::Uuid;

// 服务器配置
pub const OPEN_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 25565);
pub const PEND_TICK_TIME: Duration = Duration::from_millis(1000);  // 计划单个Tick花费的时间
// 性能调优配置
pub const WCHUNK_SIZE: usize = 16;  // 单位是 Block
pub const SCHUNK_SIZE: usize = 16;  // 单位是 WChunk
pub const MAX_THREAD_COUNT: usize = 64;
pub const CPU_USAGE_THRESHOLD: f32 = 90f32;  // 当CPU使用率超过这个值时, 会关闭线程分裂合并调控, 单位是%
pub const MAX_SCHUNK_DECAY_RATE: u8 = u8::MAX;
pub static mut GLOBAL: MaybeUninit<Global> = MaybeUninit::uninit();
pub struct Global {
    /// 异步运行时 // TODO 可能会更换
    pub runtime: Runtime,
    /// TCP监听器, 不阻塞, 轮询获取, 用于监听 JavaClient 的连接 // TODO 未来可能会使用他接收控制台命令
    pub listener: TcpListener,
    /// SChunk 脱离一个 ChunKServer 自建一个 ChunkServer 的速率
    pub schunk_decay_rate: u8,
    /// 用于主线程与 ChunkServer 的交流
    pub events: Channel<ChunkServerEvent>,
    /// 可以粗略的认为是上一个tick的耗时
    pub mspt: Duration,
    /// 以一个tick作为采样数据, 得到的所有CPU的平均使用率,
    pub avg_cpu_usage: f32,
    /// 表示一个操作系统, // TODO 因为他的一些方法开销略大,未来可能会被优化
    pub system: System,
    /// World 的 UUID 映射到 World
    pub worlds: HashMap<Uuid, World, BuildHasherDefault<XxHash64>>,
    /// 使用 Player 的 UUID 找到所在的 Chunk 然后获取可变借用, 此处 *const SyncEntityPos 管理实际的所有权
    pub players: HashMap<Uuid, SyncEntityPos, BuildHasherDefault<XxHash64>>,
    /// 当大量的 SChunk 被请求到一个 ChunkServer 时, 请求的 Chunk 会被调度到这里
    pub wchunks: DashMap<WChunkPos, RwLock<Box<SChunk>>, BuildHasherDefault<XxHash64>>,
    
}
impl Global {
    pub(crate) fn adjust_record_performance(&mut self, tick_cycle: Duration, mspt: Duration) {
        // TODO 此处最大的开销, 在 100μs 数量级, 需要优化
        // self.system.refresh_cpu_all();
        self.mspt = mspt;
        // TODO 此处其次的开销, 需要优化
        // self.avg_cpu_usage = self.system.global_cpu_usage();
        if self.avg_cpu_usage < CPU_USAGE_THRESHOLD {
            self.schunk_decay_rate = if self.mspt > tick_cycle {
                self.schunk_decay_rate.saturating_add(1)
            } else {
                self.schunk_decay_rate.saturating_sub(1)
            }
        }
    }
}
/// 在main函数中初始化, 后续只读是安全的
pub const fn global() -> &'static Global {
    unsafe {
        &*addr_of!(GLOBAL).cast::<Global>()
    }
}
pub trait AsyncWait<T> {
    fn async_wait(self) -> T;
}
impl<T, F: Future<Output = T>> AsyncWait<T> for F {
    fn async_wait(self) -> T {
        // TODO 可能最后会这么做
        // while self.poll() == Pending {
        //     // 获取任务执行
        // }
        // // 返回
        global().runtime.block_on(self)
    }
}