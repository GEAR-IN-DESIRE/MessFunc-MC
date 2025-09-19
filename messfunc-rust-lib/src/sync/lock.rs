pub trait ThreadSafe: Sync + Send {}

/// CAS锁
pub struct CASLock {

}

/// 同步互斥锁
pub struct SyncMutexLock {

}

/// 异步互斥锁
pub struct AsyncMutexLock {

}

/// 升级锁CASLock -> MutexLock
pub struct CASSyncMutexLock {

}

/// 升级锁CASLock -> AsyncMutexLock
pub struct CASAsyncMutexLock {

}

/// 同步读写锁
pub struct SyncRwLock {

}

/// 异步读写锁
pub struct AsyncRwLock {

}

/// 升级锁CASLock -> SyncRwlock
pub struct CASSyncRwLock {

}

/// 升级锁CASLock -> AsyncRwlock
pub struct CASAsyncRwLock {

}
