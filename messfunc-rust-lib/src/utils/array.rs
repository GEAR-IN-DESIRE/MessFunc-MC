pub fn create_large_array<T, const SIZE: usize>(init_fn: impl Fn() -> T) -> Box<[T; SIZE]> {
    let vec: Vec<T> = (0..SIZE).map(|_| init_fn()).collect();
    vec.into_boxed_slice().try_into().map_err(|_| "Size mismatch").unwrap()
}
