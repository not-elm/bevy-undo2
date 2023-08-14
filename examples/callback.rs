use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::math::Vec2;
use bevy::prelude::{Camera2dBundle, Color, Commands, KeyCode, Res, Sprite};
use bevy::sprite::SpriteBundle;
use bevy::utils::default;

use bevy_undo2::prelude::{UndoCallbackEventWriter, UndoRequester};
use bevy_undo2::Undo2Plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Undo2Plugin)
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_input_system)
        .run();
}


fn setup(
    mut commands: Commands,
    mut w: UndoCallbackEventWriter,
) {
    commands.spawn(Camera2dBundle::default());
    let id = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                color: Color::RED,
                ..default()
            },
            ..default()
        })
        .id();

    w.on_undo(move |cmd| {
        cmd.entity(id).despawn();
    })
}


fn keyboard_input_system(
    mut requester: UndoRequester,
    key: Res<Input<KeyCode>>,
) {
    if key.pressed(KeyCode::R) {
        requester.undo();
    }
}


