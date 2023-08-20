use bevy::app::{App, Update};
use bevy::prelude::{Event, EventReader, EventWriter, IntoSystemConfigs, ResMut};
use crate::{CommitReservationsEvent, DecrementCounterEvent, UndoRegisteredArea};
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
            register_all_reserved_events_system::<E>,
            push_undo_event_system::<E>,
            pop_undo_event_system::<E>,
            pop_undo_event_system::<E>,
            pop_undo_event_system::<UndoReserveEvent<E>>,
            reserve_event_system::<E>
        ).chain());
        self
    }
}


fn register_all_reserved_events_system<E: Event + Clone>(
    mut er: EventReader<CommitReservationsEvent>,
    mut preserve: ResMut<UndoReservedArea<E>>,
    mut registered_reserve_event_area: ResMut<UndoRegisteredArea<UndoReserveEvent<E>>>,
) {
    if let Some(CommitReservationsEvent(counter)) = er.iter().next() {
        while let Some(event) = preserve.pop() {
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


fn pop_undo_event_system<E: Event + Clone>(
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
    mut registered_area: ResMut<UndoRegisteredArea<UndoReserveEvent<E>>>,
    mut decrement_writer: EventWriter<DecrementCounterEvent>,
) {
    if er.is_empty() {
        return;
    }

    for event in er.iter() {
        ew.send(event.inner.clone());
        if event.reserve_no == 1{
            return;
        }
    }

    while let Some(event) = registered_area.pop() {
        ew.send(event.inner.clone());
        decrement_writer.send(DecrementCounterEvent);
        if event.reserve_no == 1{
            return;
        }
    }
}


