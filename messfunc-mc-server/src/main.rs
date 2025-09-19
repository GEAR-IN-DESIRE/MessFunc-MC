use crate::server::global::{global, Global, GLOBAL, MAX_THREAD_COUNT, OPEN_ADDR, PEND_TICK_TIME};
use crate::server::ticker::Ticker;
use crate::server::ChunkServer;
use dashmap::DashMap;
use messfunc_rust_lib::Channel;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::mem::MaybeUninit;
use std::net::TcpListener;
use std::ptr::addr_of_mut;
use std::time::Duration;
use sysinfo::System;
use tokio::runtime;
use tokio::time::{interval, Instant};
use twox_hash::XxHash64;

mod entity;
mod net;
mod server;
mod world;
mod block;
mod pos;

/// 如果在 mian 函数中初始化的数据, 在其他位置访问的需要初始化的数据, 应当认为一定被初始化而无需 unsafe 访问, 如 global()
fn main() {
    let inst = Instant::now();
    println!("初始化开始");
    let listener = TcpListener::bind(OPEN_ADDR).expect("端口被占了或者无效地址");
    listener.set_nonblocking(true).expect("设置TcpListener为不阻塞模式失败");
    let global = Global {
        runtime: runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(MAX_THREAD_COUNT)
            .thread_name("Thread")
            .thread_stack_size(3 * 1024 * 1024)
            .enable_all()
            .build()
            .expect("创建运行时失败"),
        listener,
        schunk_decay_rate: 0,
        events: Channel::new(),
        mspt: Duration::from_millis(0),
        avg_cpu_usage: 0f32,
        system: System::new_all(),
        worlds: HashMap::with_hasher(BuildHasherDefault::new()),
        players: HashMap::with_hasher(BuildHasherDefault::new()),
        wchunks: DashMap::with_hasher(BuildHasherDefault::new()),
    };
    unsafe {
        addr_of_mut!(GLOBAL).write(MaybeUninit::new(global));
    };
    let ticker = Ticker {
        servers: HashMap::with_hasher(BuildHasherDefault::<XxHash64>::new()),
        pendings: HashMap::with_hasher(BuildHasherDefault::<XxHash64>::new()),
        waitings: HashMap::with_hasher(BuildHasherDefault::<XxHash64>::new()),
        tasks: Vec::new(),
        interval: crate::global().runtime.block_on(async {
            interval(PEND_TICK_TIME)
        }),
        tick_start_time: Instant::now(),
        scheduled_run: true,
        schunk_pos_map: HashMap::with_hasher(BuildHasherDefault::<XxHash64>::new()),
        thread_count: 0,
        server_builder: 0,
    };
    println!("初始化结束, 耗时：{:?}", inst.elapsed());
    ticker.run();
}
/// 在 tick 开始时执行
/// 由主线程执行, 允许读写任意 WChunk + 读写 Global
pub fn start(_ticker: &mut Ticker) {
    #[cfg(debug_assertions)]
    println!("tick开始 ----------------------------------");

}
/// 由多个 Chunk线程 执行, 允许读写任意 WChunk, 以及只读 Global
pub fn tick_chunk(server: &mut ChunkServer) {
    #[cfg(debug_assertions)]
    dbg!(global().schunk_decay_rate, server.id);
}
/// 在 tick 结束时执行
/// 由主线程计算, 允许读写任意数据
pub fn end(ticker: &mut Ticker) {
    while let Some(task) = ticker.tasks.pop() {
        task(ticker)
    }
}

