use bevy::app::{App, Plugin, PostUpdate, PreUpdate};
use bevy::prelude::{Event, EventReader, IntoSystemConfigs, ResMut, Resource, resource_exists_and_equals};

use crate::counter::UndoCounter;
use crate::request::RequestUndoEvent;
use crate::undo_event::UndoEvent;

mod counter;
mod extension;
mod request;
mod undo_event;

pub mod prelude {
    pub use crate::request::UndoRequester;
    pub use crate::Undo2Plugin;
    pub use crate::extension::AppUndoEx;
    #[cfg(feature = "callback_event")]
    pub use crate::undo_event::callback::UndoCallbackEventWriter;
    pub use crate::undo_event::UndoEventWriter;
}

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Undo2Plugin;


impl Plugin for Undo2Plugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UndoEventStatus>()
            .init_resource::<UndoCounter>()
            .add_event::<RequestUndoEvent>()
            .add_systems(PreUpdate, request_undo_system)
            .add_systems(PostUpdate, decrement_count_system.run_if(resource_exists_and_equals(UndoEventStatus::Sent)));

        #[cfg(feature = "callback_event")]
        app.add_plugins(crate::undo_event::callback::UndoCallbackEventPlugin);
    }
}


#[derive(Resource, Default, PartialEq, Debug)]
enum UndoEventStatus {
    #[default]
    None,

    RequestUndo,

    Sent,
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
    mut status: ResMut<UndoEventStatus>,
) {
    if er.iter().next().is_some() {
        *status = UndoEventStatus::RequestUndo;
    }
}


fn decrement_count_system(
    mut status: ResMut<UndoEventStatus>,
    mut counter: ResMut<UndoCounter>,
) {
    *status = UndoEventStatus::None;
    counter.decrement();
}



