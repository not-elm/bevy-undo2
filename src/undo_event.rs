use bevy::ecs::system::SystemParam;
use bevy::prelude::{Event, EventWriter, ResMut};

use crate::counter::UndoCounter;
use crate::reserve::{RequestCommitReservationsEvent, RequestCommitReservationsFromSchedulerEvent, ReserveCounter, UndoReservedArea, UndoReserveEvent};

#[cfg(feature = "callback_event")]
pub mod callback;

#[derive(Event, Clone)]
pub(crate) struct UndoEvent<E: Event + Clone> {
    pub inner: E,
    pub no: usize,
}

#[derive(SystemParam)]
pub struct UndoReserveCommitter<'w> {
    ew: EventWriter<'w, RequestCommitReservationsEvent>,
}

impl<'w> UndoReserveCommitter<'w> {
    /// Moves all events placed on the reserved area by [`reserve`](UndoScheduler::reserve) to the registered area.
    #[inline(always)]
    pub fn commit(&mut self) {
        self.ew.send(RequestCommitReservationsEvent);
    }
}


#[derive(SystemParam)]
pub struct UndoScheduler<'w, E: Event + Clone> {
    counter: ResMut<'w, UndoCounter>,
    reserve: ResMut<'w, UndoReservedArea<E>>,
    reserve_counter: ResMut<'w, ReserveCounter>,
    undo_writer: EventWriter<'w, UndoEvent<E>>,
    reserve_writer: EventWriter<'w, RequestCommitReservationsFromSchedulerEvent>,
}


impl<'w, E: Event + Clone> UndoScheduler<'w, E> {
    /// Register the undo-event　in the registered area.
    ///
    /// Events can registered multiple, and when [`UndoRequester::undo`](crate::request::UndoRequester) is called,
    /// last registered will sent
    #[inline(always)]
    pub fn register(&mut self, event: E) {
        self.counter.increment();
        self.undo_writer.send(UndoEvent {
            inner: event,
            no: **self.counter,
        });
    }


    /// Place the undo-event in the reserved area.
    ///
    /// Events is  in placed on same reserved area until [`reserve_commit`](UndoScheduler::register_all_reserved) is called.
    ///
    ///
    /// This method is useful when want to sent  multiple undo-event with single call [`UndoRequest::undo`](crate::request::UndoRequester) .
    #[inline]
    pub fn reserve(&mut self, event: E) {
        self.reserve_counter.increment();
        self.reserve.push(UndoReserveEvent {
            inner: event,
            reserve_no: **self.reserve_counter,
        });
    }


    /// Moves all events placed on the reserved area by [`reserve`](UndoScheduler::reserve) to the registered area.
    #[inline]
    pub fn register_all_reserved(&mut self) {
        self.reserve_writer.send(RequestCommitReservationsFromSchedulerEvent);
    }
}


impl<'w, E: Event + Clone + Default> UndoScheduler<'w, E> {
    /// Register the undo-event　in the registered area with default value.
    ///
    /// Events can registered multiple, and when [`UndoRequester::undo`](crate::request::UndoRequester) is called,
    /// last registered will sent
    #[inline(always)]
    pub fn register_default(&mut self) {
        self.register(E::default());
    }


    /// Place the undo-event in the reserved area with default value.
    ///
    /// Events is  in placed on same reserved area until [`reserve_commit`](UndoScheduler::register_all_reserved) is called.
    ///
    ///
    /// This method is useful when want to sent  multiple undo-event with single call [`UndoRequest::undo`](crate::request::UndoRequester) .
    #[inline(always)]
    pub fn reserve_default(&mut self) {
        self.reserve(E::default());
    }
}