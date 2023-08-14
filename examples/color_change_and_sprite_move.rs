use std::time::Duration;
use bevy::app::{App, Startup, Update};
use bevy::asset::AssetServer;
use bevy::DefaultPlugins;
use bevy::hierarchy::BuildChildren;
use bevy::input::Input;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{any_with_component, Camera2dBundle, Children, Color, Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, KeyCode, Query, Res, Sprite, Text, Text2dBundle, TextStyle, Transform, With};
use bevy::sprite::SpriteBundle;
use bevy::utils::default;
use bevy_tweening::{Animator, EaseMethod, Tween, TweenCompleted, TweeningPlugin};
use bevy_tweening::lens::TransformPositionLens;

use bevy_undo2::prelude::{AppUndoEx, UndoRequester, UndoScheduler};
use bevy_undo2::UndoPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TweeningPlugin))
        .add_plugins(UndoPlugin)
        .add_undo_event::<ChangeColorEvent>()
        .add_undo_event::<UndoMoveEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            change_color_system,
            request_undo_system,
            move_system,
            undo_color_system,
            undo_move_system,
            tween_completed_system.run_if(any_with_component::<UndoMoveEvent>())
        ))
        .run();
}


#[derive(Component)]
struct ColorBox;


#[derive(Component)]
struct Movable;

#[derive(Event, Copy, Clone)]
struct ChangeColorEvent(Color);

#[derive(Event, Copy, Clone, Component)]
struct UndoMoveEvent(Vec3);

fn setup(mut commands: Commands, asset: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                color: Color::RED,
                ..default()
            },
            transform: Transform::from_xyz(-300., 0., 0.),
            ..default()
        })
        .insert(ColorBox);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })

        .insert(Movable)
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section("Stop", TextStyle {
                    font: asset.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 31.,
                    color: Color::BLACK,
                }),
                transform: Transform::from_xyz(0., 0., 1.),
                ..default()
            });
        });
}


fn request_undo_system(
    mut requester: UndoRequester,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::R) {
        requester.undo();
    }
}


fn change_color_system(
    mut scheduler: UndoScheduler<ChangeColorEvent>,
    mut color_box: Query<&mut Sprite, With<ColorBox>>,
    key: Res<Input<KeyCode>>,
) {
    let mut color_box = color_box.single_mut();
    let mut register = move |new_color: Color| {
        let color = color_box.color;
        color_box.color = new_color;
        scheduler.register(ChangeColorEvent(color));
    };

    if key.just_pressed(KeyCode::Key1) {
        register(Color::RED);
    } else if key.just_pressed(KeyCode::Key2) {
        register(Color::GREEN);
    } else if key.just_pressed(KeyCode::Key3) {
        register(Color::BLUE);
    }
}


fn move_system(
    mut commands: Commands,
    mut text: Query<&mut Text>,
    mov: Query<(Entity, &Transform, &Children), With<Movable>>,
    key: Res<Input<KeyCode>>,
) {
    let (me, mt, children) = mov.single();

    let start = mt.translation;
    let mut start_move = |message: &str, relative: Vec3| {
        let tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_millis(500),
            TransformPositionLens {
                start,
                end: start + relative * 100.,
            },
        )
            .with_completed_event(0);

        text.get_mut(*children.first().unwrap()).unwrap().sections[0].value = message.to_string();
        commands.entity(me).insert(Animator::new(tween)).insert(UndoMoveEvent(start));
    };

    if key.just_pressed(KeyCode::Left) {
        start_move("Left", Vec3::NEG_X);
    } else if key.just_pressed(KeyCode::Up) {
        start_move("Up", Vec3::Y);
    } else if key.just_pressed(KeyCode::Right) {
        start_move("Right", Vec3::X);
    } else if key.just_pressed(KeyCode::Down) {
        start_move("Down", Vec3::NEG_Y);
    }
}


fn tween_completed_system(
    mut commands: Commands,
    mut er: EventReader<TweenCompleted>,
    mut scheduler: UndoScheduler<UndoMoveEvent>,
    mut text: Query<&mut Text>,
    mov: Query<(Entity, &Children, &UndoMoveEvent), With<Movable>>,
) {
    for _ in er.iter() {
        let (me, children, undo_event) = mov.single();
        text.get_mut(*children.first().unwrap()).unwrap().sections[0].value = "Stop".to_string();
        commands.entity(me).remove::<UndoMoveEvent>();
        scheduler.register(*undo_event);
    }
}


fn undo_move_system(
    mut er: EventReader<UndoMoveEvent>,
    mut mov: Query<&mut Transform, With<Movable>>,
) {
    for e in er.iter() {
        mov.single_mut().translation = e.0;
    }
}


fn undo_color_system(
    mut er: EventReader<ChangeColorEvent>,
    mut color_box: Query<&mut Sprite, With<ColorBox>>,
) {
    let Some(ChangeColorEvent(color)) = er.iter().next() else { return; };
    color_box.single_mut().color = *color;
}