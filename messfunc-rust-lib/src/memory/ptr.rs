#[inline(always)]
pub const unsafe fn ptr_to_mut<'a, T>(ptr: *const T) -> &'a mut T {
    unsafe { &mut *(ptr as *mut T) }
}

#[inline(always)]
pub const unsafe fn ptr_to_ref<'a, T>(ptr: *const T) -> &'a T {
    unsafe { &*(ptr as *const T) }
}
