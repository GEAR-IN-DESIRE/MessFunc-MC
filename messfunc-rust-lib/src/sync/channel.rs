use std::cell::UnsafeCell;
use std::ptr;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct Channel<T> {
    lock: AtomicBool,
    notify: Notify,
    inner: UnsafeCell<Vec<T>>,
}

impl<T> Channel<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            lock: AtomicBool::new(false),
            notify: Notify::new(),
            inner: UnsafeCell::new(Vec::new()),
        }
    }

    pub async fn send(&self, val: T) {
        loop {
            if self.lock.compare_exchange_weak(
                false,
                true,
                Acquire,
                Relaxed,
            ).is_ok() {
                unsafe {
                    (*self.inner.get()).push(val);
                }
                self.lock.store(false, Release);
                self.notify.notify_one();
                return;
            } else {
                self.notify.notified().await;
            }
        }
    }
    
    pub async fn wait_recv(&self) -> T {
        loop {
            if self.lock.compare_exchange_weak(
                false,
                true,
                Acquire,
                Relaxed,
            ).is_ok() {
                let result = unsafe {
                    (*self.inner.get()).pop()
                };
                self.lock.store(false, Release);

                if let Some(val) = result {
                    return val;
                } else {
                    self.notify.notified().await;
                }
            } else {
                self.notify.notified().await;
            }
        }
    }
}

unsafe  impl<T: Send> Send for Channel<T> {}
unsafe  impl<T: Sync> Sync for Channel<T> {}


/// TODO 可以固定为 OnetimeReceiver 释放内存
pub struct OnetimeChannel<T: Send> {
    ready: AtomicBool,
    notify: Notify,
    val: UnsafeCell<Option<T>>,
}

impl<T: Send> OnetimeChannel<T> {
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            notify: Notify::new(),
            val: UnsafeCell::new(None),
        }
    }
    
    #[inline(always)]
    pub fn split(self) -> (OnetimeSender<T>, OnetimeReceiver<T>) {
        let channel = Arc::new(self);
        let tx = OnetimeSender { channel: channel.clone() };
        let rx = OnetimeReceiver { channel };
        (tx, rx)
    }

    /// TODO 需要限制只能调用一次
    pub fn send(&self, val: T) {
        unsafe {
            ptr::write(self.val.get(), Some(val));
        }
        self.ready.store(true, Release);
        self.notify.notify_waiters();
    }

    pub async fn wait_recv(&self) -> T {
        loop {
            if self.ready.load(Acquire) {
                unsafe {
                    break ptr::replace(self.val.get(), None).unwrap_unchecked();
                }
            } else {
                self.notify.notified().await;
            }
        }
    }
}

unsafe impl<T: Send> Send for OnetimeChannel<T> {}
unsafe impl<T: Send> Sync for OnetimeChannel<T> {}

pub struct OnetimeSender<T: Send> {
    channel: Arc<OnetimeChannel<T>>,
}

impl<T: Send> OnetimeSender<T> {
    #[inline(always)]
    pub fn send(self, val: T) {
        self.channel.send(val);
    }
}

#[derive(Clone)]
pub struct OnetimeReceiver<T: Send> {
    channel: Arc<OnetimeChannel<T>>,
}

impl<T: Send> OnetimeReceiver<T> {
    #[inline(always)]
    pub async fn wait_recv(self) -> T {
        self.channel.wait_recv().await
    }
}
