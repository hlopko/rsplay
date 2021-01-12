use std::mem;

use crate::unsafe_cell::UnsafeCell;
pub struct Cell<T> {
    value: UnsafeCell<T>,
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, value: T) {
        unsafe {
            // SAFETY: There are no other references to the UnsafeCell,
            // it's safe to get a mutable reference (it will be unique) and mutate through it.
            let mut cell = &mut *self.value.get();
            *cell = value;
        }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe {
            // SAFETY: There are no other references to the UnsafeCell,
            // it's safe to access the contents and create a copy from it.
            return *self.value.get();
        }
    }

    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.replace(Default::default())
    }

    pub fn replace(&self, val: T) -> T {
        mem::replace(
            unsafe {
                // SAFETY: Cell is not Sync, so there are no other concurrent mutations possible.
                &mut *self.value.get()
            },
            val,
        )
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_cell_ref_can_get_copy() {
        let cell = Cell::new(42);
        let (p1, p2): (&Cell<i32>, &Cell<i32>) = (&cell, &cell);
        p1.set(52);

        assert_eq!(cell.get(), 52);
        assert_eq!(p1.get(), 52);
        assert_eq!(p2.get(), 52);
    }

    #[test]
    fn shared_cell_can_take_and_leave_default() {
        struct Thingy(i32);
        impl Default for Thingy {
            fn default() -> Self {
                Thingy(42)
            }
        }

        let cell = Cell::new(Thingy(12));
        let (p1, p2) = (&cell, &cell);

        assert_eq!(p1.take().0, 12);
        assert_eq!(p2.take().0, 42);
        assert_eq!(cell.take().0, 42);
    }

    #[test]
    fn shared_cell_can_replace() {        
        let cell = Cell::new(42);
        let (p1, p2) = (&cell, &cell);

        assert_eq!(p1.replace(43), 42);
        assert_eq!(p2.replace(44), 43);
        assert_eq!(cell.replace(45), 44);
    }

    #[test]
    fn mutable_cell_ref_can_get_mut() {
        let mut cell = Cell::new(42);

        let p = &mut cell;

        assert_eq!(p.get_mut(), &42);
        assert_eq!(cell.get_mut(), &42);
    }

    #[test]
    fn owned_cell_can_implode_with_into_inner() {        
        let cell = Cell::new(42);
       
        assert_eq!(cell.into_inner(), 42);
    }
}
