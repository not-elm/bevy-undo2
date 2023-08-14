use std::ops::Deref;

use bevy::prelude::Resource;

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


impl Deref for UndoCounter {
    type Target = usize;


    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}