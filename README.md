# bevy-undo2

This crate makes it easy to use the undo-operation on [bevy](https://bevyengine.org/).

[![License: MIT/Apache](https://img.shields.io/badge/License-MIT%20or%20Apache2-blue.svg)](https://opensource.org/licenses/MIT)
[![Crate](https://img.shields.io/crates/v/bevy-undo2.svg)](https://crates.io/crates/bevy-undo2)

## Examples

[examples/simple.rs](./examples/simple.rs)

```rust

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
    asset: Res<AssetServer>
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(Text2dBundle{
        text: Text::from_section("Please Press [R]", TextStyle{
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
    mut text: Query<&mut Text>
) {
    for GreetEvent(message) in er.iter() {
        text.single_mut().sections[0].value = message.clone();
    }
}
```

### Callback

Callbacks can also be registered by using `UndoCallbackEvent`, which is built in by default.

[examples/callback.rs](./examples/callback.rs)

```rust
fn setup(
    mut scheduler: UndoScheduler<UndoCallbackEvent>
) {
    let entity: Enity;
    scheduler.register(UndoCallbackEvent::new(move |cmd| {
        cmd.entity(text).despawn();
    }));
}
```

### Reserved Area

It is possible to send multiple events with single call `undo` by placing in the reserved area.

See below for an example:

[examples/reserve.rs](./examples/reserve.rs)


## Compatible Bevy versions

| this crate | bevy   |
|------------|--------|
| 0.0.1      | 0.11.0 |
