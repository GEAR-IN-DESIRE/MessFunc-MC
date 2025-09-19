use std::sync::atomic::Ordering::{AcqRel, Acquire};
use std::sync::atomic::{AtomicPtr, AtomicU64};

///   AtomicBox  ///
#[derive(Debug)]
pub struct AtomicBox<T> {
    // TODO 虽然有限的时间里几乎不可能溢出, 但是不是不能, 这种逻辑上的不完美令我感到很难受
    version: AtomicU64,
    // 他应该在内部保证不可能为null
    ptr: AtomicPtr<T>,
}
impl<T> AtomicBox<T> {
    pub fn new(val: T) -> Self {
        Self {
            version: AtomicU64::new(0),
            ptr: AtomicPtr::new(Box::into_raw(Box::new(val))),
        }
    }
    pub fn update(&self, val: T) -> Box<T> {
        // 必须在 swap 之前, 确保版本变化先于指针更新被其他线程看到
        self.version.fetch_add(1, AcqRel);
        unsafe {
            Box::from_raw(
                self.ptr.swap(
                    Box::into_raw(Box::new(val)),
                    AcqRel
                )
            )
        }
    }
}

impl<T: Copy> AtomicBox<T> {
    pub fn load(&self) -> T {
        loop {
            let old = self.version.load(Acquire);
            let value = unsafe { *self.ptr.load(Acquire) };
            if self.version.load(Acquire) == old {
                return value
            }
        }
    }
}

impl<T> From<T> for AtomicBox<T> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<T> Drop for AtomicBox<T> {
    fn drop(&mut self) {
        let ptr = self.ptr.load(Acquire);
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

unsafe impl<T: Sync> Sync for AtomicBox<T> {}
unsafe impl<T: Send> Send for AtomicBox<T> {}
