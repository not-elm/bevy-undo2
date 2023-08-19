use bevy::prelude::*;
use bevy_undo2::prelude::*;


#[derive(Event, Debug, Clone)]
enum UndoColorEvent {
    Red,
    Blue,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UndoPlugin)
        .add_undo_event::<UndoColorEvent>()
        .add_systems(Update, (
            reserve_red_system,
            reserve_blue_system,
            request_undo_system,
            register_all_reserved_system,
            undo_system
        ))
        .run();
}


fn reserve_red_system(
    mut scheduler: UndoScheduler<UndoColorEvent>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Key1) {
        println!("Reserved {:?}", UndoColorEvent::Red);
        scheduler.reserve(UndoColorEvent::Red);
    }
}


fn reserve_blue_system(
    mut scheduler: UndoScheduler<UndoColorEvent>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Key2) {
        println!("Reserved {:?}", UndoColorEvent::Blue);
        scheduler.reserve(UndoColorEvent::Blue);
    }
}


fn register_all_reserved_system(
    mut committer: UndoReserveCommitter,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Key3) {
        println!("Register all reserved");
        committer.commit();
    }
}


fn request_undo_system(
    mut requester: UndoRequester,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::R) {
        requester.undo();
    }
}


fn undo_system(mut er: EventReader<UndoColorEvent>) {
    for event in er.iter() {
        println!("{:?}", event);
    }
}