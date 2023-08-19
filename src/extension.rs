use bevy::app::{App, FixedUpdate, Update};
use bevy::prelude::{Event, EventReader, EventWriter, ResMut};
use crate::{CommitReservationsEvent, DecrementCounterEvent, UndoStack};
use crate::prelude::{UndoRequester, UndoScheduler};
use crate::request::RequestUndoEvent;
use crate::reserve::{ReserveCounter, UndoReserve, UndoReserveEvent};
use crate::undo_event::UndoEvent;

pub trait AppUndoEx {
    /// Setup the app to manage undo events of type `T`
    ///
    /// In order to use undo-action, you must call [`UndoScheduler::register`](UndoScheduler::register).
    /// then call [`UndoRequester::undo`](UndoRequester::undo) when you need.
    fn add_undo_event<T: Event + Clone>(&mut self) -> &mut App;
}


impl AppUndoEx for App {
    fn add_undo_event<E: Event + Clone>(&mut self) -> &mut App {
        self.add_event::<E>();
        self.add_event::<UndoEvent<E>>();
        self.add_event::<UndoReserveEvent<E>>();
        self.init_resource::<UndoStack<E>>();
        self.init_resource::<UndoStack<UndoReserveEvent<E>>>();
        self.init_resource::<UndoReserve<E>>();
        self.init_resource::<ReserveCounter>();
        self.add_systems(Update, (
            commit_reservations_system::<E>,
            pop_undo_event_system::<E>,
            pop_undo_event_system::<E>,
            pop_undo_event_system::<UndoReserveEvent<E>>,
            push_undo_event_system::<E>
        ));
        self.add_systems(FixedUpdate, reserve_event_system::<E>);
        self
    }
}


fn pop_undo_event_system<E: Event + Clone>(
    mut er: EventReader<RequestUndoEvent>,
    mut ew: EventWriter<E>,
    mut decrement_writer: EventWriter<DecrementCounterEvent>,
    mut undo_stack: ResMut<UndoStack<E>>,
) {
    for RequestUndoEvent(counter) in er.iter() {
        if let Some(undo) = undo_stack.pop_if_has_latest(counter) {
            ew.send(undo);
            decrement_writer.send(DecrementCounterEvent);
        }
    }
}


fn push_undo_event_system<E: Event + Clone>(
    mut er: EventReader<UndoEvent<E>>,
    mut undo_stack: ResMut<UndoStack<E>>,
) {
    for e in er.iter() {
        undo_stack.push(e.clone());
    }
}


fn commit_reservations_system<E: Event + Clone>(
    mut er: EventReader<CommitReservationsEvent>,
    mut preserve: ResMut<UndoReserve<E>>,
    mut undo_stack: ResMut<UndoStack<UndoReserveEvent<E>>>,
) {
    if let Some(CommitReservationsEvent(counter)) = er.iter().next() {
        while let Some(event) = preserve.pop() {
            undo_stack.push(UndoEvent {
                inner: event.clone(),
                no: **counter + event.reserve_no,
            });
        }
    }
}


fn reserve_event_system<E: Event + Clone>(
    mut er: EventReader<UndoReserveEvent<E>>,
    mut ew: EventWriter<E>,
    mut requester: UndoRequester,
) {
    for e in er.iter() {
        ew.send(e.inner.clone());

        if 1 < e.reserve_no {
            requester.undo();
        }
    }
}


