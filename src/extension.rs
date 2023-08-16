use bevy::app::{App, FixedUpdate, Update};
use bevy::prelude::{Event, EventReader, EventWriter, in_state, IntoSystemConfigs, Res, ResMut};

use crate::{Posted, UndoStack, UndoState};
use crate::counter::UndoCounter;
use crate::prelude::UndoRequester;
use crate::reserve::{ReserveCounter, UndoReserve, UndoReserveEvent};
use crate::undo_event::UndoEvent;

pub trait AppUndoEx {
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
        self.add_systems(FixedUpdate, commit_reservations_system::<E>
            .run_if(in_state(UndoState::CommitReservations)),
        );
        self.add_systems(Update, (
            pop_undo_event_system::<E>,
            pop_undo_event_system::<UndoReserveEvent<E>>
        )
            .run_if(in_state(UndoState::RequestUndo)),
        );
        self.add_systems(FixedUpdate, (push_undo_event_system::<E>, reserve_event_system::<E>));
        self
    }
}


fn pop_undo_event_system<E: Event + Clone>(
    mut ew: EventWriter<E>,
    mut undo_stack: ResMut<UndoStack<E>>,
    mut posted: ResMut<Posted>,
    counter: Res<UndoCounter>,
) {
    if let Some(undo) = undo_stack.pop_if_has_latest(&counter) {
        posted.0 = true;
        ew.send(undo);
    }
}


fn commit_reservations_system<E: Event + Clone>(
    mut preserve: ResMut<UndoReserve<E>>,
    mut undo_stack: ResMut<UndoStack<UndoReserveEvent<E>>>,
    counter: Res<UndoCounter>,
) {
    while let Some(event) = preserve.pop() {
        undo_stack.push(UndoEvent {
            inner: event.clone(),
            no: **counter + event.reserve_no,
        });
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


