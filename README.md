# Aseprite plugin for Bevy

[![latest]][crates.io] [![doc]][docs.rs]

[latest]: https://img.shields.io/crates/v/bevy_mod_aseprite.svg
[crates.io]: https://crates.io/crates/bevy_mod_aseprite
[doc]: https://docs.rs/bevy_mod_aseprite/badge.svg
[docs.rs]: https://docs.rs/bevy_mod_aseprite

A plugin for using [Aseprite][] animations in [Bevy][].

[bevy]: https://bevyengine.org/
[aseprite]: https://www.aseprite.org/

## Usage

Basic usage:

```rust,ignore
commands.spawn(AsepriteBundle {
    aseprite: asset_server.load("player.ase"),
    animation: AsepriteAnimation::from("stand"),
    transform: TransformBundle::default(),
});
```

Using `aseprite!` macro for compile time validation:

```rust,ignore
mod sprites {
  use bevy_mod_aseprite::aseprite;
  aseprite!(pub Player, "player.ase");
}

commands.spawn(AsepriteBundle {
    aseprite: asset_server.load(sprites::Player::PATH),
    animation: AsepriteAnimation::from(sprites::Player::tags::STAND),
    transform: TransformBundle::default(),
});
```

The component `AsepriteAnimation` also exposes methods to get information such as the
current animation frame (within the tag or not), its duration, or the number of
remaining frames. This can be useful to transition states at the end of an animation:

```rust,ignore
fn transition_player(
    time: Res<Time>,
    player_q: Query<(&PlayerState, &Handle<Aseprite>, &AsepriteAnimation), With<Player>>,
    aseprites: Res<Assets<Aseprite>>,
    mut ev_player_changed: EventWriter<PlayerChanged>,
) {
    let (&player_state, handle, anim) = player_q.single();
    let Some(aseprite) = aseprites.get(handle) else { return };
    match player_state {
        PlayerState::Attack => {
            let remaining_frames = anim.remaining_tag_frames(aseprite.info());
            let frame_finished = anim.frame_finished(aseprite.info(), time.delta());
            if remaining_frames == 0 && frame_finished {
                ev_player_changed.send(PlayerChanged::default().new_state(PlayerState::Stand));
            }
        }
        _ => (),
    }
}
```

## Bevy Compatibility

| **bevy** | **bevy_mod_aseprite** |
|----------|-----------------------|
| 0.8      | 0.1                   |

## History

This crate started as a fork of [mdenchev/bevy_aseprite][].

[mdenchev/bevy_aseprite]: https://github.com/mdenchev/bevy_aseprite
