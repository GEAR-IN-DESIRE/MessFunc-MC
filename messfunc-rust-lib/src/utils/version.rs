#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    inner: u64,
}
impl Version {
    pub fn new() -> Self {
        Version {
            inner: 0,
        }
    }

    pub fn next(&mut self) -> Self {
        self.inner = self.inner.checked_add(1).expect("版本号自增时超出最大范围");
        Version {
            inner: self.inner
        }
    }
}