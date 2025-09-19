use std::ptr;

#[inline(always)]
pub fn take<T, R>(mut_ref: &mut T, f: impl FnOnce(T) -> (T, R)) -> R {
    unsafe {
        let (t, r) = f(ptr::read(mut_ref));
        ptr::write(mut_ref, t);
        r
    }
}
