use bevy::ecs::system::SystemParam;
use bevy::prelude::{Event, EventWriter,  ResMut};

use crate::counter::UndoCounter;
use crate::reserve::{RequestPreserveCommitEvent, ReserveCounter, UndoReserve, UndoReserveEvent};

#[cfg(feature = "callback_event")]
pub mod callback;

#[derive(Event, Clone)]
pub(crate) struct UndoEvent<E: Event + Clone> {
    pub inner: E,
    pub no: usize,
}





#[derive(SystemParam)]
pub struct UndoScheduler<'w, E: Event + Clone> {
    counter: ResMut<'w, UndoCounter>,
    reserve: ResMut<'w, UndoReserve<E>>,
    reserve_counter: ResMut<'w, ReserveCounter>,
    undo_writer: EventWriter<'w, UndoEvent<E>>,

    preserve_writer: EventWriter<'w, RequestPreserveCommitEvent>,
}


impl<'w, E: Event + Clone> UndoScheduler<'w, E> {
    #[inline(always)]
    pub fn register(&mut self, event: E) {
        self.counter.increment();
        self.undo_writer.send(UndoEvent {
            inner: event,
            no: **self.counter,
        });
    }


    #[inline]
    pub fn reserve(&mut self, event: E) {
        self.reserve_counter.increment();
        self.reserve.push(UndoReserveEvent {
            inner: event,
            reserve_no: **self.reserve_counter,
        });
    }


    #[inline]
    pub fn reserve_commit(&mut self) {
        self.preserve_writer.send(RequestPreserveCommitEvent);
    }
}


impl<'w, E: Event + Clone + Default> UndoScheduler<'w, E> {
    #[inline(always)]
    pub fn register_default(&mut self) {
        self.register(E::default());
    }


    #[inline(always)]
    pub fn reserve_default(&mut self) {
        self.reserve(E::default());
    }
}