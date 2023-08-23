use bevy::app::{App, Plugin};
use bevy::prelude::{Event, IntoSystemConfigs, EventReader, EventWriter, PreUpdate, ResMut, Resource};

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
            .add_systems(PreUpdate, (
                decrement_counter,
                reserve_reset_system
            ).chain());

        #[cfg(feature = "callback_event")]
        app.add_plugins(crate::undo_event::callback::UndoCallbackEventPlugin);
    }
}


#[derive(Resource)]
struct UndoRegisteredArea<T: Event + Clone>(Vec<UndoEvent<T>>);


impl<T: Event + Clone> Default for UndoRegisteredArea<T> {
    #[inline(always)]
    fn default() -> Self {
        Self(vec![])
    }
}


impl<E: Event + Clone> UndoRegisteredArea<E> {
    #[inline(always)]
    pub fn push(&mut self, e: UndoEvent<E>) {
        self.0.push(e);
    }


    #[inline(always)]
    pub fn pop(&mut self) -> Option<E> {
        self.0.pop().map(|e| e.inner)
    }


    #[inline(always)]
    pub fn pop_if_has_latest(&mut self, counter: &UndoCounter) -> Option<E> {
        let index = self.0.iter().position(|undo| {
            **counter <= undo.no
        })?;

        Some(self.0.remove(index).inner)
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


#[cfg(test)]
mod tests {
    use bevy::app::{App, Startup, Update};
    use bevy::input::Input;
    use bevy::prelude::{Commands, Component, Event, EventReader, KeyCode, Res};
    use crate::counter::UndoCounter;
    use crate::extension::AppUndoEx;
    use crate::prelude::UndoRequester;
    use crate::reserve::{ReserveCounter, UndoReservedArea, UndoReserveEvent};
    use crate::undo_event::UndoScheduler;
    use crate::{UndoPlugin, UndoRegisteredArea};

    #[derive(Event, Clone, Default)]
    struct UndoEvent;

    #[derive(Component)]
    struct OnUndo;


    #[test]
    fn once_register() {
        let mut app = new_app();
        app.add_systems(Startup, |mut s: UndoScheduler<UndoEvent>| {
            s.register_default();
        });
        app.update();

        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::R);
        app.update();
        app.update();

        assert_eq!(app.world.query::<&OnUndo>().iter(&app.world).len(), 1);
    }


    #[test]
    fn register_2times() {
        let mut app = new_app();
        app.add_systems(Startup, |mut s: UndoScheduler<UndoEvent>| {
            s.register_default();
            s.register_default();
        });

        app.update();

        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::R);
        app.update();

        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::R);
        app.update();

        assert_eq!(app.world.query::<&OnUndo>().iter(&app.world).len(), 1);
        app.update();

        assert_eq!(app.world.query::<&OnUndo>().iter(&app.world).len(), 2);
    }


    #[test]
    fn reserve_init_3times() {
        let mut app = new_app();
        app.add_systems(Startup, |mut s: UndoScheduler<UndoEvent>| {
            s.reserve_default();
            s.reserve_default();
            s.reserve_default();
            s.register_all_reserved();
        });

        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::R);
        app.update();
        app.update();

        assert_eq!(app.world.query::<&OnUndo>().iter(&app.world).len(), 3);
    }


    #[test]
    fn reserve_at_intervals() {
        let mut app = new_app();
        app.add_systems(Update, |mut s: UndoScheduler<UndoEvent>, key: Res<Input<KeyCode>>| {
            if key.just_pressed(KeyCode::A) {
                s.reserve_default();
            } else if key.just_pressed(KeyCode::B) {
                s.register_all_reserved();
            }
        });

        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::A);
        app.update();
        assert_eq!(app.world.resource_mut::<UndoReservedArea<UndoEvent>>().0.len(), 1);

        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::A);
        app.update();
        assert_eq!(app.world.resource_mut::<UndoReservedArea<UndoEvent>>().0.len(), 2);

        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::A);
        app.update();
           assert_eq!(app.world.resource_mut::<UndoReservedArea<UndoEvent>>().0.len(), 3);

        app.world.resource_mut::<Input<KeyCode>>().reset(KeyCode::A);
        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::B);
        app.update();
        app.world.resource_mut::<Input<KeyCode>>().reset(KeyCode::B);
        app.update();

        assert_eq!(app.world.resource_mut::<UndoReservedArea<UndoEvent>>().0.len(), 0);
        assert_eq!(app.world.resource_mut::<UndoRegisteredArea<UndoReserveEvent<UndoEvent>>>().0.len(), 3);

        app.world.resource_mut::<Input<KeyCode>>().reset(KeyCode::B);
        app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::R);
        app.update();

        app.update();

        assert_eq!(**app.world.resource_mut::<UndoCounter>(), 0);
        assert_eq!(**app.world.resource_mut::<ReserveCounter>(), 0);
        assert_eq!(app.world.resource_mut::<UndoRegisteredArea<UndoEvent>>().0.len(), 0);
        assert_eq!(app.world.resource_mut::<UndoReservedArea<UndoEvent>>().0.len(), 0);
        assert_eq!(app.world.query::<&OnUndo>().iter(&app.world).len(), 3);
    }


    fn undo(mut req: UndoRequester, key: Res<Input<KeyCode>>) {
        if key.just_pressed(KeyCode::R) {
            req.undo();
        }
    }

    fn read_undo(
        mut commands: Commands,
        mut er: EventReader<UndoEvent>,
    ) {
        for _ in er.iter() {
            commands.spawn(OnUndo);
        }
    }

    fn new_app() -> App {
        let mut app = App::new();
        app.add_plugins(UndoPlugin);
        app.init_resource::<Input<KeyCode>>();
        app.add_undo_event::<UndoEvent>();
        app.add_systems(Update, read_undo);
        app.add_systems(Update, undo);

        app
    }
}