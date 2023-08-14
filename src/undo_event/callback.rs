use std::sync::Arc;

use bevy::app::{App, Plugin, Update};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Commands, Event, EventReader};

use crate::extension::AppUndoEx;
use crate::prelude::UndoEventWriter;

#[derive(SystemParam)]
pub struct UndoCallbackEventWriter<'w>(UndoEventWriter<'w, UndoCallbackEvent>);

impl<'w> UndoCallbackEventWriter<'w> {
    #[inline(always)]
    pub fn on_undo(&mut self, f: impl Fn(&mut Commands) + Send + Sync + 'static) {
        self.0.write(UndoCallbackEvent::new(f));
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


#[cfg(test)]
mod tests {
    use bevy::app::{App, Startup, Update};
    use bevy::prelude::Component;

    use crate::Undo2Plugin;
    use crate::prelude::UndoRequester;
    use crate::undo_event::callback::UndoCallbackEventWriter;

    #[test]
    fn a() {
        let mut app = App::new();
        app.add_plugins(Undo2Plugin);
        app.add_systems(Startup, on_undo);
        app.add_systems(Update, undo);
        app.update();
        // assert!(app.world.query::<&Test>().iter(&app.world).len() == 1);
    }

    #[derive(Component)]
    struct Test;

    fn undo(mut req: UndoRequester) {
        req.undo();
    }

    fn on_undo(mut w: UndoCallbackEventWriter) {
        w.on_undo(|cmd| {
            cmd.spawn(Test);
        });
    }
}