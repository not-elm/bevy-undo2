use bevy::app::{App, Update};
use bevy::prelude::{Event, EventReader, EventWriter, in_state, IntoSystemConfigs, Res, ResMut};

use crate::{SentUndo, UndoEventState, UndoStack};
use crate::counter::UndoCounter;
use crate::undo_event::UndoEvent;

pub trait AppUndoEx {
    fn add_undo_event<T: Event + Clone>(&mut self) -> &mut App;
}


impl AppUndoEx for App {
    fn add_undo_event<T: Event + Clone>(&mut self) -> &mut App {
        self.add_event::<T>();
        self.add_event::<UndoEvent<T>>();
        self.init_resource::<UndoStack<T>>();
        self.add_systems(Update, pop_undo_event_system::<T>
            .run_if(in_state(UndoEventState::RequestUndo)),
        );
        self.add_systems(Update, push_undo_event_system::<T>);
        self
    }
}


fn pop_undo_event_system<E: Event + Clone>(
    mut ew: EventWriter<E>,
    mut sent_event: ResMut<SentUndo>,
    mut undo_stack: ResMut<UndoStack<E>>,
    counter: Res<UndoCounter>,
) {
    while let Some(undo) = undo_stack.pop_if_has_latest(&counter) {
        sent_event.0 = true;
        ew.send(undo);
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