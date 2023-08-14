use bevy::app::{App, Plugin, PostUpdate, PreUpdate};
use bevy::prelude::{Event, EventReader, in_state, IntoSystemConfigs, NextState, ResMut, Resource, States};

use crate::counter::UndoCounter;
use crate::request::RequestUndoEvent;
use crate::undo_event::UndoEvent;

mod counter;
mod extension;
mod request;
mod undo_event;

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
            .init_resource::<UndoCounter>()
            .init_resource::<SentUndo>()
            .add_event::<RequestUndoEvent>()
            .add_systems(PreUpdate, request_undo_system)
            .add_systems(PostUpdate, decrement_count_system.run_if(in_state(UndoEventState::RequestUndo)));

        #[cfg(feature = "callback_event")]
        app.add_plugins(crate::undo_event::callback::UndoCallbackEventPlugin);
    }
}


#[derive(States, Default, PartialEq, Debug, Copy, Clone, Eq, Hash)]
enum UndoEventState {
    #[default]
    None,

    RequestUndo,
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


impl<E: Event + Clone> UndoStack<E> {
    #[inline(always)]
    pub fn push(&mut self, e: UndoEvent<E>) {
        self.0.push(e);
    }


    #[inline(always)]
    pub fn pop_if_has_latest(&mut self, counter: &UndoCounter) -> Option<E> {
        if self.need_pop(counter) {
            self.0.pop().map(|e| e.inner)
        } else {
            None
        }
    }


    #[inline(always)]
    fn need_pop(&self, counter: &UndoCounter) -> bool {
        self.0.last().is_some_and(|undo| undo.no == **counter)
    }
}


fn request_undo_system(
    mut er: EventReader<RequestUndoEvent>,
    mut state: ResMut<NextState<UndoEventState>>,
) {
    if er.iter().next().is_some() {
        state.set(UndoEventState::RequestUndo);
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



