use bevy::ecs::system::SystemParam;
use bevy::prelude::{Event, EventWriter, ResMut};

use crate::counter::UndoCounter;

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
    ew: EventWriter<'w, UndoEvent<E>>,
}


impl<'w, E: Event + Clone> UndoScheduler<'w, E> {
    #[inline(always)]
    pub fn register(&mut self, event: E) {
        self.counter.increment();
        self.ew.send(UndoEvent {
            inner: event,
            no: **self.counter,
        });
    }
}


impl<'w, E: Event + Clone + Default> UndoScheduler<'w, E> {
    #[inline(always)]
    pub fn register_default(&mut self) {
        self.register(E::default());
    }
}