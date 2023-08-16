use bevy::prelude::*;
use bevy_undo2::prelude::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UndoPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_input_system)
        .run();
}


fn setup(
    mut commands: Commands,
    mut scheduler: UndoScheduler<UndoCallbackEvent>,
    asset: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    let text = commands
        .spawn(Text2dBundle {
            text: Text::from_section("Please Press [R]: Delete text", TextStyle {
                font: asset.load("fonts/FiraSans-Bold.ttf"),
                font_size: 31.,
                ..default()
            }),
            ..default()
        })
        .id();

    scheduler.register(UndoCallbackEvent::new(move |cmd| {
        cmd.entity(text).despawn();
    }));
}


fn keyboard_input_system(
    mut requester: UndoRequester,
    key: Res<Input<KeyCode>>,
) {
    if key.pressed(KeyCode::R) {
        requester.undo();
    }
}


