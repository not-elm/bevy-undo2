use bevy::app::{App, FixedUpdate, Plugin, PostUpdate, Update};
use bevy::prelude::{Event, EventReader, EventWriter, ResMut, Resource};

use crate::counter::UndoCounter;
use crate::request::RequestUndoEvent;
use crate::reserve::{RequestCommitReservationsEvent, RequestCommitReservationsFromSchedulerEvent, ReserveCounter};
use crate::undo_event::UndoEvent;

mod counter;
mod extension;
mod request;
mod undo_event;
mod reserve;

pub mod prelude {
    pub use crate::extension::AppUndoEx;
    pub use crate::request::{UndoRequester};
    pub use crate::undo_event::{UndoReserveCommitter, UndoScheduler};
    #[cfg(feature = "callback_event")]
    pub use crate::undo_event::callback::UndoCallbackEvent;
    pub use crate::UndoPlugin;
}


/// Add undo-operations to an app.
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Hash)]
pub struct UndoPlugin;


impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RequestUndoEvent>()
            .add_event::<CommitReservationsEvent>()
            .add_event::<DecrementCounterEvent>()
            .add_event::<RequestCommitReservationsFromSchedulerEvent>()
            .add_event::<RequestCommitReservationsEvent>()
            .init_resource::<UndoCounter>()
            .add_systems(Update, (
                decrement_counter,
                reserve_reset_system
            ));

        #[cfg(feature = "callback_event")]
        app.add_plugins(crate::undo_event::callback::UndoCallbackEventPlugin);
    }
}


#[derive(Resource)]
struct UndoStack<T: Event + Clone>(Vec<UndoEvent<T>>);


impl<T: Event + Clone> Default for UndoStack<T> {
    #[inline(always)]
    fn default() -> Self {
        Self(vec![])
    }
}


impl<E: Event + Clone> UndoStack<E> {
    #[inline(always)]
    pub fn push(&mut self, e: UndoEvent<E>) {
        self.0.push(e);
    }


    #[inline(always)]
    pub fn pop_if_has_latest(&mut self, counter: &UndoCounter) -> Option<E> {
        self.0.iter().for_each(|undo| println!("{counter:?} {}", undo.no));
        if self.0.last().is_some_and(|undo| undo.no == **counter) {
            self.0.pop().map(|undo| undo.inner)
        } else {
            None
        }
    }
}


#[derive(Event)]
pub(crate) struct DecrementCounterEvent;


fn decrement_counter(
    mut er: EventReader<DecrementCounterEvent>,
    mut counter: ResMut<UndoCounter>,
) {
    for _ in er.iter() {
        counter.decrement();
    }
}


#[derive(Event)]
pub(crate) struct CommitReservationsEvent(pub UndoCounter);

fn reserve_reset_system(
    mut er: EventReader<RequestCommitReservationsEvent>,
    mut er2: EventReader<RequestCommitReservationsFromSchedulerEvent>,
    mut ew: EventWriter<CommitReservationsEvent>,
    mut counter: ResMut<UndoCounter>,
    mut reserve_counter: ResMut<ReserveCounter>,
) {
    if er.iter().next().is_some() || er2.iter().next().is_some() {
        ew.send(CommitReservationsEvent(*counter));
        *counter += *reserve_counter;
        reserve_counter.reset();
    }
}