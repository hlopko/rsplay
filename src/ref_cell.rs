use std::fmt::{self, Debug, Display};

use crate::{cell::Cell, refs::RefMut};
use crate::refs::{Ref, State};
use crate::unsafe_cell::UnsafeCell;

pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<State>,
}

/// An error returned by [`RefCell::try_borrow`].
pub struct BorrowError {}

impl Debug for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BorrowError").finish()
    }
}

impl Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Already mutably borrowed").finish()
    }
}

/// An error returned by [`RefCell::try_borrow`].
pub struct BorrowMutError {}

impl Debug for BorrowMutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BorrowMutError").finish()
    }
}

impl Display for BorrowMutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Already borrowed").finish()
    }
}

impl<T> RefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: Cell::new(State::Unused),
        }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    // pub fn replace(&self, newval: T) -> T {
    //     self.value.replace(newval)
    // }

    pub fn borrow(&self) -> Ref<T> {
        self.try_borrow()
            .expect("Value borrowed mutably, can't borrow.")
    }

    pub fn try_borrow(&self) -> Result<Ref<T>, BorrowError> {
        match self.state.get() {
            State::Unused => {
                self.state.set(State::HasReaders(1));
                Ok(unsafe {
                    // SAFETY: This is safe because we have no pending borrows.
                    Ref::new(&self.state, &*self.value.get())
                })
            }
            State::HasReaders(n) => {
                self.state.set(State::HasReaders(n + 1));
                Ok(unsafe {
                    // SAFETY: This is safe because we have only have readers.
                    Ref::new(&self.state, &*self.value.get())
                })
            }
            State::HasWriter => Err(BorrowError {}),
        }
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        match self.state.get() {
            State::Unused => {
                self.state.set(State::HasWriter);
                Ok(unsafe {
                    // SAFETY: This is safe because we have no pending borrows.
                    RefMut::new(&self.state, &mut *self.value.get())
                })
            }
            State::HasReaders(_) => Err(BorrowMutError {}),
            State::HasWriter => Err(BorrowMutError {}),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.try_borrow_mut().expect("Value already borrowed")
    }
}

impl<T> Debug for RefCell<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_borrow() {
            Ok(borrow) => f.debug_struct("RefCell").field("value", &borrow).finish(),
            Err(_) => {
                struct Placeholder;

                impl Debug for Placeholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<borrowed>")
                    }
                }

                f.debug_struct("RefCell")
                    .field("value", &Placeholder)
                    .finish()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_inner() {
        let cell = RefCell::new(42);
        assert_eq!(cell.into_inner(), 42);
    }

    // #[test]
    // fn test_replace() {
    //     let cell = RefCell::new(5);
    //     let old_value = cell.replace(6);
    //     assert_eq!(old_value, 5);
    //     assert_eq!(cell, RefCell::new(6));
    // }

    #[test]
    fn test_borrow() {
        let cell = RefCell::new(42);

        let b1 = cell.borrow();
        let b2 = cell.borrow();

        assert_eq!(*b1, 42);
        assert_eq!(*b2, 42);
    }
    #[test]
    fn test_try_borrow_ok_for_unused() {
        let cell = RefCell::new(42);
        assert!(cell.try_borrow().is_ok());
    }
    #[test]
    fn test_try_borrow_ok_for_readers() {
        let cell = RefCell::new(42);
        let _ = cell.borrow();
        assert!(cell.try_borrow().is_ok());
    }

    #[test]
    fn test_try_borrow_is_err_when_writers() {
        let cell = RefCell::new(42);

        let _mut_borrow = cell.borrow_mut();
        assert!(cell.try_borrow().is_err());
    }

    #[test]
    fn test_try_borrow_mut_ok_for_unused() {
        let cell = RefCell::new(42);
        assert!(cell.try_borrow_mut().is_ok());
    }
    #[test]
    fn test_try_borrow_mut_is_err_when_readers() {
        let cell = RefCell::new(42);
        let _borrow = cell.borrow();
        assert!(cell.try_borrow_mut().is_err());
    }

    #[test]
    fn test_try_borrow_mut_is_err_when_writers() {
        let cell = RefCell::new(42);

        let _mut_borrow = cell.borrow_mut();
        assert!(cell.try_borrow_mut().is_err());
    }

    #[test]
    fn test_borrow_error_debug() {
        let err = BorrowError {};
        assert_eq!(format!("{:?}", err), "BorrowError");
    }

    #[test]
    fn test_borrow_error_display() {
        let err = BorrowError {};
        assert_eq!(format!("{}", err), "Already mutably borrowed");
    }

    #[test]
    fn test_borrow_mut_error_debug() {
        let err = BorrowMutError {};
        assert_eq!(format!("{:?}", err), "BorrowMutError");
    }

    #[test]
    fn test_borrow_mut_error_display() {
        let err = BorrowMutError {};
        assert_eq!(format!("{}", err), "Already borrowed");
    }

    #[test]
    fn test_impl_debug() {
        let cell = RefCell::new(42);
        let _borrow = &cell;
        assert_eq!(format!("{:?}", cell), "RefCell { value: Ref { value: 42 } }");
    }

    #[test]
    fn test_impl_debug_borrowed() {
        let cell = RefCell::new(42);
        let _mut_borrow = cell.borrow_mut();
        assert_eq!(format!("{:?}", cell), "RefCell { value: <borrowed> }");
    }

    #[test]
    fn test_transition_from_readers_to_writers() {
        let cell = RefCell::new(42);
        {
            let _scoped_borrow = cell.borrow();
        }

        assert!(cell.try_borrow_mut().is_ok());
    }

    #[test]
    fn test_transition_from_writers_to_readers() {
        let cell = RefCell::new(42);
        {
            let _scoped_mut_borrow = cell.borrow_mut();
        }

        assert!(cell.try_borrow().is_ok());
    }
}
