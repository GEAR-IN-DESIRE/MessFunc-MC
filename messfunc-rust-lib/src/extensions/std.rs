///  Option
pub trait OptionExt<T> {
    fn with_some(self, f: impl FnOnce(T)) -> bool;
    fn with_some_ref(&self, f: impl FnOnce(&T)) -> bool;
    fn with_some_mut(&mut self, f: impl FnOnce(&mut T)) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn with_some(self, f: impl FnOnce(T)) -> bool {
        match self {
            None => false,
            Some(t) => {
                f(t);
                true
            }
        }
    }

    fn with_some_ref(&self, f: impl FnOnce(&T)) -> bool {
        match self {
            None => false,
            Some(t) => {
                f(t);
                true
            }
        }
    }

    fn with_some_mut(&mut self, f: impl FnOnce(&mut T)) -> bool {
        match self {
            None => false,
            Some(t) => {
                f(t);
                true
            }
        }
    }
}
