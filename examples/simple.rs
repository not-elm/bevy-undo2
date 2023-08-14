use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::prelude::{Event, EventReader, KeyCode, Res};

use bevy_undo2::prelude::{AppUndoEx, UndoEventWriter, UndoRequester};
use bevy_undo2::Undo2Plugin;

#[derive(Event, Clone)]
struct GreetEvent(String);


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Undo2Plugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_input_system, read_undo_event_system))
        .add_undo_event::<GreetEvent>()
        .run();
}


fn setup(
    mut w: UndoEventWriter<GreetEvent>
) {
    w.write(GreetEvent("Hello World!".to_string()));
}


fn keyboard_input_system(
    mut requester: UndoRequester,
    key: Res<Input<KeyCode>>,
) {
    if key.pressed(KeyCode::R) {
        requester.undo();
    }
}


fn read_undo_event_system(mut er: EventReader<GreetEvent>) {
    for GreetEvent(message) in er.iter() {
        println!("{message}");
    }
}