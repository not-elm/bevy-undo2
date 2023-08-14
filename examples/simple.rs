use bevy::prelude::*;
use bevy_undo2::prelude::*;

#[derive(Event, Clone)]
struct GreetEvent(String);


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UndoPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_input_system, read_undo_event_system))
        .add_undo_event::<GreetEvent>()
        .run();
}


fn setup(
    mut commands: Commands,
    mut scheduler: UndoScheduler<GreetEvent>,
    asset: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(Text2dBundle {
        text: Text::from_section("Please Press [R]", TextStyle {
            font: asset.load("fonts/FiraSans-Bold.ttf"),
            font_size: 31.,
            ..default()
        }),
        ..default()
    });
    scheduler.register(GreetEvent("Undo!".to_string()));
}


fn keyboard_input_system(
    mut requester: UndoRequester,
    key: Res<Input<KeyCode>>,
) {
    if key.pressed(KeyCode::R) {
        requester.undo();
    }
}


fn read_undo_event_system(
    mut er: EventReader<GreetEvent>,
    mut text: Query<&mut Text>,
) {
    for GreetEvent(message) in er.iter() {
        text.single_mut().sections[0].value = message.clone();
    }
}