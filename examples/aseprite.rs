use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::utils::HashSet;
use bevy_mod_aseprite::{Aseprite, AsepriteAnimation, AsepriteBundle, AsepritePlugin, AsepriteTag};

pub mod sprites {
    use bevy_mod_aseprite::aseprite;
    aseprite!(pub Player, "player.ase");
}

pub fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(AsepritePlugin)
        .init_resource::<Events<PlayerChanged>>()
        .add_startup_system(setup)
        .add_system(keyboard_input)
        .add_system(transition_player)
        .add_system(update_player)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn()
        .insert(Player)
        .insert(PlayerState::Stand)
        .insert(Orientation::Right)
        .insert_bundle(AsepriteBundle {
            aseprite: asset_server.load(sprites::Player::PATH),
            animation: AsepriteAnimation::from(sprites::Player::tags::STAND),
            transform: TransformBundle::from_transform(Transform {
                scale: Vec3::splat(1.5),
                ..default()
            }),
        });
}

fn update_player(
    time: Res<Time>,
    mut commands: Commands,
    mut ev_player_changed: EventReader<PlayerChanged>,
    mut player_q: Query<
        (
            Entity,
            &mut PlayerState,
            Option<&Movements>,
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut AsepriteAnimation,
            &mut Orientation,
        ),
        With<Player>,
    >,
) {
    let Ok((
        player,
        mut player_state,
        movements,
        mut transform,
        mut sprite_atlas,
        mut animation,
        mut orientation
    )) = player_q.get_single_mut() else { return };

    for PlayerChanged {
        new_state,
        new_orientation,
        new_movements,
    } in ev_player_changed.iter()
    {
        if let Some(new_state) = new_state {
            *animation = AsepriteAnimation::from(new_state.animation_tag());
            match new_state {
                PlayerState::Stand | PlayerState::Attack => {
                    commands.entity(player).remove::<Movements>();
                }
                _ => (),
            }
            *player_state = *new_state;
        }

        if let Some(new_orientation) = new_orientation {
            sprite_atlas.flip_x = new_orientation.flip_x();
            *orientation = *new_orientation;
        }

        if let Some(new_movements) = new_movements {
            commands.entity(player).insert(new_movements.clone());
        }
    }

    if let Some(movements) = movements {
        for moving in movements.iter() {
            let velocity = 150.;
            match moving {
                Moving::Left => transform.translation.x -= velocity * time.delta_seconds(),
                Moving::Right => transform.translation.x += velocity * time.delta_seconds(),
                Moving::Up => transform.translation.y += velocity * time.delta_seconds(),
                Moving::Down => transform.translation.y -= velocity * time.delta_seconds(),
            }
        }
    }
}

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

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    player_q: Query<&PlayerState, With<Player>>,
    mut ev_player_changed: EventWriter<PlayerChanged>,
) {
    let player_state = player_q.single();

    if keyboard_direction_pressed(&keys) && !keyboard_attack_detected(&keys) {
        match *player_state {
            PlayerState::Attack => {
                return;
            }
            PlayerState::Move => {
                let movements = Movements::from_keyboard(&keys);
                let new_orientation = Orientation::from_movements(&movements);
                ev_player_changed.send(
                    PlayerChanged::default()
                        .new_movements(movements)
                        .new_orientation(new_orientation),
                );
            }
            PlayerState::Stand => {
                ev_player_changed.send(
                    PlayerChanged::default()
                        .new_state(PlayerState::Move)
                        .new_movements(Movements::from_keyboard(&keys)),
                );
            }
        }
    } else if keyboard_direction_just_released(&keys) {
        match *player_state {
            PlayerState::Move => (),
            _ => return,
        }
        ev_player_changed.send(PlayerChanged::default().new_state(PlayerState::Stand));
    } else if keyboard_attack_detected(&keys) {
        match *player_state {
            PlayerState::Attack => return,
            _ => (),
        }
        ev_player_changed.send(PlayerChanged::default().new_state(PlayerState::Attack));
    }
}

fn keyboard_direction_pressed(keys: &Input<KeyCode>) -> bool {
    keys.pressed(KeyCode::Left)
        || keys.pressed(KeyCode::Up)
        || keys.pressed(KeyCode::Down)
        || keys.pressed(KeyCode::Right)
}

fn keyboard_direction_just_released(keys: &Input<KeyCode>) -> bool {
    keys.just_released(KeyCode::Left)
        || keys.just_released(KeyCode::Up)
        || keys.just_released(KeyCode::Down)
        || keys.just_released(KeyCode::Right)
}

fn keyboard_attack_detected(keys: &Input<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::Space)
}

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Component, Debug, Clone, Copy)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy)]
pub enum PlayerState {
    Stand,
    Move,
    Attack,
}

impl PlayerState {
    fn animation_tag(&self) -> AsepriteTag {
        match &self {
            Self::Stand => sprites::Player::tags::STAND,
            Self::Move => sprites::Player::tags::MOVE,
            Self::Attack => sprites::Player::tags::ATTACK,
        }
        .into()
    }
}

#[derive(Default)]
pub struct PlayerChanged {
    new_state: Option<PlayerState>,
    new_orientation: Option<Orientation>,
    new_movements: Option<Movements>,
}

impl PlayerChanged {
    pub fn new_state<N: Into<Option<PlayerState>>>(mut self, new_state: N) -> Self {
        self.new_state = new_state.into();
        self
    }

    pub fn new_orientation<N: Into<Option<Orientation>>>(mut self, new_orientation: N) -> Self {
        self.new_orientation = new_orientation.into();
        self
    }

    pub fn new_movements<N: Into<Option<Movements>>>(mut self, new_movements: N) -> Self {
        self.new_movements = new_movements.into();
        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Moving {
    Left,
    Up,
    Down,
    Right,
}

#[derive(Component, Deref, Debug, Clone)]
pub struct Movements(pub HashSet<Moving>);

impl Movements {
    pub fn from_keyboard(keys: &Input<KeyCode>) -> Self {
        let mut movements = HashSet::with_capacity(4);
        if keys.pressed(KeyCode::Left) {
            movements.insert(Moving::Left);
        }
        if keys.pressed(KeyCode::Up) {
            movements.insert(Moving::Up);
        }
        if keys.pressed(KeyCode::Down) {
            movements.insert(Moving::Down);
        }
        if keys.pressed(KeyCode::Right) {
            movements.insert(Moving::Right);
        }
        Self(movements)
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub enum Orientation {
    Left,
    Right,
}

impl Orientation {
    pub fn from_movements(movements: &Movements) -> Option<Self> {
        if movements.contains(&Moving::Left) && !movements.contains(&Moving::Right) {
            Some(Orientation::Left)
        } else if movements.contains(&Moving::Right) && !movements.contains(&Moving::Left) {
            Some(Orientation::Right)
        } else {
            None
        }
    }

    pub fn flip_x(&self) -> bool {
        match self {
            Self::Right => false,
            Self::Left => true,
        }
    }
}
