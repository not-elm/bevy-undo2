use bevy::prelude::*;
use bevy_undo2::prelude::*;


#[derive(Event, Debug, Clone)]
struct UndoEvent1(String);


#[derive(Resource, Debug, Clone, Default)]
struct Count(usize);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Count>()
        .add_plugins(UndoPlugin)
        .add_undo_event::<UndoEvent1>()
        .add_systems(Update, (keyboard_input_system, undo_system))
        .run();
}


fn keyboard_input_system(
    mut u1: UndoScheduler<UndoEvent1>,
    mut requester: UndoRequester,
    mut count: ResMut<Count>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Up) {
        match count.0 {
            0 => {
                println!("Reserve 0");
                u1.reserve(UndoEvent1("Undo 0".to_string()));
            }
            1 => {
                println!("Reserve 1");
                u1.reserve(UndoEvent1("Undo 1".to_string()));
            }
            2 => {
                println!("Reserve 2");
                u1.reserve(UndoEvent1("Undo 2".to_string()));
                u1.reserve_commit();
            }
            _ => {}
        }

        count.0 = (count.0 + 1) % 3;
    } else if key.just_pressed(KeyCode::R) {
        requester.undo();
    }
}


fn undo_system(mut er: EventReader<UndoEvent1>) {
    for UndoEvent1(message) in er.iter() {
        println!("{}", message);
    }
}