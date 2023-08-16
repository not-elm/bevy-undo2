use std::ops::{AddAssign, Deref};

use bevy::prelude::Resource;
use crate::reserve::ReserveCounter;

#[derive(Resource, Default, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub(crate) struct UndoCounter(usize);


impl UndoCounter {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0)
    }


    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 1;
    }


    #[inline(always)]
    pub fn decrement(&mut self) {
        self.0 = self.0.checked_sub(1).unwrap_or_default();
    }
}



impl AddAssign<ReserveCounter> for UndoCounter{
    fn add_assign(&mut self, rhs: ReserveCounter) {
        self.0 += *rhs;
    }
}


impl Deref for UndoCounter {
    type Target = usize;


    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}