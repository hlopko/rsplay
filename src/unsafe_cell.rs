// #[repr(transparent)]
pub struct UnsafeCell<T> {
    value: T,
}

impl<T> UnsafeCell<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn get(&self) -> *mut T {
        self as *const UnsafeCell<T> as *const T as *mut T
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_inner() {
        let cell = UnsafeCell::new(42);
        assert_eq!(cell.into_inner(), 42);
    }

    #[test]
    fn test_get() {
        let cell = UnsafeCell::new(42);
        assert_eq!(unsafe { *cell.get() }, 42);
    }

    #[test]
    fn test_get_mut() {
        let mut cell = UnsafeCell::new(42);
        let inner = cell.get_mut();
        *inner = 43;
        assert_eq!(unsafe { *cell.get() }, 43);
    }
}
