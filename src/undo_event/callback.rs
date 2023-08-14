use std::sync::Arc;

use bevy::app::{App, Plugin, Update};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Commands, Event, EventReader};

use crate::extension::AppUndoEx;
use crate::prelude::UndoScheduler;

#[derive(SystemParam)]
pub struct UndoCallbackScheduler<'w>(UndoScheduler<'w, UndoCallbackEvent>);

impl<'w> UndoCallbackScheduler<'w> {
    #[inline(always)]
    pub fn register(&mut self, f: impl Fn(&mut Commands) + Send + Sync + 'static) {
        self.0.register(UndoCallbackEvent::new(f));
    }
}


#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub(crate) struct UndoCallbackEventPlugin;

impl Plugin for UndoCallbackEventPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app
            .add_undo_event::<UndoCallbackEvent>()
            .add_systems(Update, undo_callback_event_system);
    }
}


#[derive(Event, Clone)]
pub(crate) struct UndoCallbackEvent(Arc<dyn Fn(&mut Commands) + Send + Sync + 'static>);


impl UndoCallbackEvent {
    #[inline(always)]
    pub fn new(f: impl Fn(&mut Commands) + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }
}


#[inline]
pub(crate) fn undo_callback_event_system(
    mut commands: Commands,
    mut er: EventReader<UndoCallbackEvent>,
) {
    for e in er.iter() {
        e.0(&mut commands);
    }
}


