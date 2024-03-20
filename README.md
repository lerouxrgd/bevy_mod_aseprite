# Aseprite plugin for Bevy

[![latest]][crates.io] [![doc]][docs.rs]

[latest]: https://img.shields.io/crates/v/bevy_mod_aseprite.svg
[crates.io]: https://crates.io/crates/bevy_mod_aseprite
[doc]: https://docs.rs/bevy_mod_aseprite/badge.svg
[docs.rs]: https://docs.rs/bevy_mod_aseprite

A plugin for using [Aseprite][] animations in [Bevy][].

The [`AsepriteBundle`][aseprite-bundle] is composed of the same fields as the
[`SpriteSheetBundle`][spritesheet-bundle] but with two extra components,
[`Handle<Aseprite>`][aseprite-handle] and [`AsepriteAnimation`][aseprite-anim].

[bevy]: https://bevyengine.org/
[aseprite]: https://www.aseprite.org/
[spritesheet-bundle]: https://docs.rs/bevy/latest/bevy/prelude/struct.SpriteSheetBundle.html
[aseprite-bundle]: https://docs.rs/bevy_mod_aseprite/latest/bevy_mod_aseprite/struct.AsepriteBundle.html
[aseprite-handle]: https://docs.rs/bevy_mod_aseprite/latest/bevy_mod_aseprite/struct.Aseprite.html
[aseprite-anim]: https://docs.rs/bevy_mod_aseprite/latest/bevy_mod_aseprite/struct.AsepriteAnimation.html

## Example

<p align="center">
    <img src="assets/player.gif" width="395" height="250" alt="Aseprite Example" />
</p>

See [examples/aseprite.rs][example-aseprite] for a complete example, you can run it with:

```ignore
cargo run --example aseprite
```

[example-aseprite]: https://github.com/lerouxrgd/bevy_mod_aseprite/blob/master/examples/aseprite.rs

## Usage

Basic usage is as follows:

```rust,ignore
fn load_assets(asset_server: Res<AssetServer>, mut aseprite_handles: ResMut<AsepriteHandles>) {
    let player: Handle<Aseprite> = asset_server.load("player.ase");
    aseprite_handles.push(player);
}

fn setup(
    mut commands: Commands,
    aseprite_handles: Res<AsepriteHandles>,
    aseprites: Res<Assets<Aseprite>>,
) {
    let aseprite_handle = &aseprite_handles[0];
    let aseprite = aseprites.get(aseprite_handle).unwrap();
    let animation = AsepriteAnimation::new(aseprite.info(), "idle");

    commands
        .spawn(Player)
        .insert(AsepriteBundle {
            texture: aseprite.texture().clone_weak(),
            atlas: TextureAtlas {
                index: animation.current_frame(),
                layout: aseprite.layout().clone_weak(),
            },
            aseprite: aseprite_handle.clone_weak(),
            animation,
            ..default()
        });
}

#[derive(Resource, Deref, DerefMut, Default)]
struct AsepriteHandles(Vec<Handle<Aseprite>>);
```

The component [`AsepriteAnimation`][aseprite-anim] also exposes methods to get
information such as the current animation frame (within the tag or not), its duration,
or the number of remaining frames. This can be useful to transition states at the end of
an animation:

```rust,ignore
fn transition_player(
    time: Res<Time>,
    player_q: Query<(&PlayerState, &Handle<Aseprite>, &AsepriteAnimation), With<Player>>,
    aseprites: Res<Assets<Aseprite>>,
    mut ev_player_changed: EventWriter<PlayerChanged>,
) {
    let (&player_state, handle, anim) = player_q.single();
    let aseprite = aseprites.get(handle).unwrap();
    match player_state {
        PlayerState::Attack => {
            let remaining_frames = anim.remaining_tag_frames(aseprite.info()).unwrap();
            let frame_finished = anim.frame_finished(time.delta());
            if remaining_frames == 0 && frame_finished {
                ev_player_changed.send(PlayerState::Stand.into());
            }
        }
        _ => (),
    }
}
```

## Bevy Compatibility

| **bevy** | **bevy_mod_aseprite** |
|----------|-----------------------|
| 0.13     | 0.7                   |
| 0.12     | 0.6                   |
| 0.11     | 0.5                   |
| 0.10     | 0.4                   |
| 0.9      | 0.2, 0.3              |
| 0.8      | 0.1                   |

## History

This crate started as a fork of [mdenchev/bevy_aseprite][].

[mdenchev/bevy_aseprite]: https://github.com/mdenchev/bevy_aseprite
