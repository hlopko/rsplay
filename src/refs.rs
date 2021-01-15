use std::{borrow::{Borrow, BorrowMut}, ops::{Deref, DerefMut}};
use std::fmt::Debug;

use crate::cell::Cell;

#[derive(Clone, Copy)]
pub enum State {
    Unused,
    HasReaders(usize),
    HasWriter,
}
pub struct Ref<'cell, T> {
    state: &'cell Cell<State>,
    value: &'cell T
}

impl<'cell, T> Ref<'cell, T> {
    pub fn new(state: &'cell Cell<State>, value: &'cell T) -> Self {
        Self { state, value }
    }
}

impl<'cell, T> Drop for Ref<'cell, T> {
    fn drop(&mut self) {
        let state = self.state.get();
        if let State::HasReaders(n) = state {
            if n == 1 {
                self.state.set(State::Unused);
            } else {
                self.state.set(State::HasReaders(n - 1));
            }
        } else {
            unreachable!("Cannot have a Ref instance when the cell is not in the HasReaders state");
        }
    }
}

impl<T> Borrow<T> for Ref<'_, T> {
    fn borrow(&self) -> &T {
        self.value
    }
}

impl<T: Debug> Debug for Ref<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ref").field("value", self.value).finish()
    }
}

impl<T: PartialEq> PartialEq for Ref<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(other.value)
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct RefMut<'cell, T> {
    state: &'cell Cell<State>,
    value: &'cell mut T
}

impl<'cell, T> RefMut<'cell, T> {
    pub fn new(state: &'cell Cell<State>, value: &'cell mut T) -> Self {
        Self { state, value }
    }
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        let state = self.state.get();
        if let State::HasWriter = state {
            self.state.set(State::Unused);
        } else {
            unreachable!("Cannot have a RefMut instance when the cell is not in the HasWriter state");
        }
    }
}

impl<T> Borrow<T> for RefMut<'_, T> {
    fn borrow(&self) -> &T {
        self.value
    }
}

impl<T> BorrowMut<T> for RefMut<'_, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}