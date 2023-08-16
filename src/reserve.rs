use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::Deref;
use bevy::prelude::{Event, Resource};


#[derive(Event, Clone)]
pub(crate) struct RequestCommitReservationsFromSchedulerEvent;


#[derive(Event, Clone)]
pub(crate) struct RequestCommitReservationsEvent;


#[derive(Event, Clone)]
pub(crate) struct UndoReserveEvent<E: Event + Clone> {
    pub inner: E,
    pub reserve_no: usize,
}


#[derive(Resource, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
#[repr(transparent)]
pub(crate) struct ReserveCounter(usize);


impl Deref for ReserveCounter {
    type Target = usize;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl ReserveCounter {
    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 1;
    }


    #[inline(always)]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}


#[derive(Resource)]
pub(crate) struct UndoReserve<E: Event + Clone>(VecDeque<UndoReserveEvent<E>>);


impl<E: Event + Clone> UndoReserve<E> {
    #[inline]
    pub fn push(&mut self, event: UndoReserveEvent<E>) {
        self.0.push_back(event);
    }


    #[inline]
    pub fn pop(&mut self) -> Option<UndoReserveEvent<E>> {
        self.0.pop_front()
    }
}


impl<E: Event + Clone> Default for UndoReserve<E> {
    #[inline(always)]
    fn default() -> Self {
        Self(VecDeque::new())
    }
}




