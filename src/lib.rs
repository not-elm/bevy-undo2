use bevy::app::{App, Plugin, PostUpdate, PreUpdate};
use bevy::prelude::{Event, EventReader, EventWriter, in_state, IntoSystemConfigs, NextState, ResMut, Resource, States};

use crate::counter::UndoCounter;
use crate::request::RequestUndoEvent;
use crate::reserve::{RequestPreserveCommitEvent, ReserveCounter};
use crate::undo_event::UndoEvent;

mod counter;
mod extension;
mod request;
mod undo_event;
mod reserve;

pub mod prelude {
    pub use crate::request::UndoRequester;
    pub use crate::UndoPlugin;
    pub use crate::extension::AppUndoEx;
    #[cfg(feature = "callback_event")]
    pub use crate::undo_event::callback::UndoCallbackScheduler;
    pub use crate::undo_event::UndoScheduler;
}


/// Add undo-operations to an app.
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Hash)]
pub struct UndoPlugin;


impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<UndoEventState>()
            .add_event::<RequestUndoEvent>()
            .add_event::<RequestPreserveCommitEvent>()
            .add_event::<UndoWaitEvent>()
            .init_resource::<UndoCounter>()
            .init_resource::<SentUndo>()
            .add_systems(PreUpdate, (request_undo_system, undo_wait_event_system).chain())
            .add_systems(PostUpdate, decrement_count_system.run_if(in_state(UndoEventState::RequestUndo)))
            .add_systems(PostUpdate, reserve_reset_system.run_if(in_state(UndoEventState::PreserveCommit)));

        #[cfg(feature = "callback_event")]
        app.add_plugins(crate::undo_event::callback::UndoCallbackEventPlugin);
    }
}


#[derive(States, Default, PartialEq, Debug, Copy, Clone, Eq, Hash)]
enum UndoEventState {
    #[default]
    None,

    RequestUndo,

    PreserveCommit,
}


#[derive(Resource)]
struct UndoStack<T: Event + Clone>(Vec<UndoEvent<T>>);


impl<T: Event + Clone> Default for UndoStack<T> {
    #[inline(always)]
    fn default() -> Self {
        Self(vec![])
    }
}


#[derive(Resource, Default)]
pub(crate) struct SentUndo(pub bool);


#[derive(Event)]
struct UndoWaitEvent;


impl<E: Event + Clone> UndoStack<E> {
    #[inline(always)]
    pub fn push(&mut self, e: UndoEvent<E>) {
        self.0.push(e);
    }


    #[inline(always)]
    pub fn pop_if_has_latest(&mut self, counter: &UndoCounter) -> Option<E> {
        let index = self.0.iter().position(|undo| undo.no == **counter)?;
        Some(self.0.remove(index).inner)
    }
}

fn request_undo_system(
    mut preserve_reader: EventReader<RequestPreserveCommitEvent>,
    mut undo_reader: EventReader<RequestUndoEvent>,
    mut wait: EventWriter<UndoWaitEvent>,
    mut state: ResMut<NextState<UndoEventState>>,
) {
    if preserve_reader.iter().next().is_some() {
        state.set(UndoEventState::PreserveCommit);
        if undo_reader.iter().next().is_some() {
            wait.send(UndoWaitEvent);
        }
    } else if undo_reader.iter().next().is_some() {
        state.set(UndoEventState::RequestUndo);
    }
}


fn undo_wait_event_system(
    mut er: EventReader<UndoWaitEvent>,
    mut ew: EventWriter<RequestUndoEvent>,
) {
    if er.iter().next().is_some() {
        ew.send(RequestUndoEvent);
    }
}


fn decrement_count_system(
    mut state: ResMut<NextState<UndoEventState>>,
    mut counter: ResMut<UndoCounter>,
    mut sent: ResMut<SentUndo>,
) {
    state.set(UndoEventState::None);
    if sent.0 {
        counter.decrement();
        sent.0 = false;
    }
}


fn reserve_reset_system(
    mut state: ResMut<NextState<UndoEventState>>,
    mut counter: ResMut<UndoCounter>,
    mut preserve_counter: ResMut<ReserveCounter>,
) {
    *counter += *preserve_counter;
    preserve_counter.reset();
    state.set(UndoEventState::None);
}