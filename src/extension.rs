use bevy::app::{App, Update};
use bevy::prelude::{Event, EventReader, EventWriter, IntoSystemConfigs, ResMut};
use crate::{CommitReservationsEvent, DecrementCounterEvent, UndoRegisteredArea};
use crate::prelude::UndoRequester;
use crate::request::RequestUndoEvent;
use crate::reserve::{ReserveCounter, UndoReservedArea, UndoReserveEvent};
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
        self.init_resource::<UndoRegisteredArea<E>>();
        self.init_resource::<UndoRegisteredArea<UndoReserveEvent<E>>>();
        self.init_resource::<UndoReservedArea<E>>();
        self.init_resource::<ReserveCounter>();
        self.add_systems(Update, (
               reserve_event_system::<E>,
            register_all_reserved_events_system::<E>,
            push_undo_event_system::<E>,
            request_undo_event_system::<E>,
            request_undo_event_system::<E>,
            request_undo_event_system::<UndoReserveEvent<E>>,

        ).chain());
        self
    }
}


fn register_all_reserved_events_system<E: Event + Clone>(
    mut er: EventReader<CommitReservationsEvent>,
    mut reserved_area: ResMut<UndoReservedArea<E>>,
    mut registered_reserve_event_area: ResMut<UndoRegisteredArea<UndoReserveEvent<E>>>,
) {
    while let Some(CommitReservationsEvent(counter)) = er.iter().next() {
        reserved_area.0.sort_by(|e1, e2| e2.reserve_no.partial_cmp(&e1.reserve_no).unwrap());

        while let Some(event) = reserved_area.pop_front() {
            registered_reserve_event_area.push(UndoEvent {
                inner: event.clone(),
                no: **counter + event.reserve_no,
            });
        }
    }
}


fn push_undo_event_system<E: Event + Clone>(
    mut er: EventReader<UndoEvent<E>>,
    mut registered_area: ResMut<UndoRegisteredArea<E>>,
) {
    for e in er.iter() {
        registered_area.push(e.clone());
    }
}


fn request_undo_event_system<E: Event + Clone>(
    mut er: EventReader<RequestUndoEvent>,
    mut ew: EventWriter<E>,
    mut decrement_writer: EventWriter<DecrementCounterEvent>,
    mut registered_area: ResMut<UndoRegisteredArea<E>>,
) {
    for RequestUndoEvent(counter) in er.iter() {
        while let Some(undo) = registered_area.pop_if_has_latest(counter) {
            ew.send(undo);
            decrement_writer.send(DecrementCounterEvent);
        }
    }
}


fn reserve_event_system<E: Event + Clone>(
    mut er: EventReader<UndoReserveEvent<E>>,
    mut ew: EventWriter<E>,
    mut undo_event_writer: UndoRequester,
) {
    if er.is_empty() {
        return;
    }

    for event in er.iter() {
        ew.send(event.inner.clone());
        if 1 < event.reserve_no {
            undo_event_writer.undo();
        }
    }
}


