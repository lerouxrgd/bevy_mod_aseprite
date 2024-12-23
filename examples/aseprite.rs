use bevy::utils::HashSet;
use bevy::{asset::LoadState, prelude::*};
use bevy_mod_aseprite::{
    Aseprite, AsepriteAnimation, AsepriteAsset, AsepritePlugin, AsepriteSystems, AsepriteTag,
};

pub mod sprites {
    use bevy_mod_aseprite::aseprite;
    aseprite!(pub Player, "player.ase");
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(AsepritePlugin)
        .init_resource::<Events<PlayerChanged>>()
        .init_resource::<AsepriteHandles>()
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Loading), load_assets)
        .add_systems(Update, check_assets.run_if(in_state(AppState::Loading)))
        .add_systems(OnExit(AppState::Loading), setup)
        .add_systems(
            Update,
            (
                keyboard_input,
                transition_player.before(AsepriteSystems::Animate),
                update_player,
            )
                .run_if(in_state(AppState::Ready)),
        )
        .run();
}

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    Loading,
    Ready,
}

#[derive(Resource, Deref, DerefMut, Default)]
struct AsepriteHandles(Vec<Handle<AsepriteAsset>>);

fn load_assets(asset_server: Res<AssetServer>, mut ase_handles: ResMut<AsepriteHandles>) {
    let player = asset_server.load(sprites::Player::PATH);
    ase_handles.push(player);
}

fn check_assets(
    ase_handles: ResMut<AsepriteHandles>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<AppState>>,
) {
    ase_handles
        .iter()
        .all(|handle| {
            matches!(
                asset_server.get_load_state(handle.id()),
                Some(LoadState::Loaded)
            )
        })
        .then(|| state.set(AppState::Ready));
}

fn setup(
    mut commands: Commands,
    ase_handles: Res<AsepriteHandles>,
    ase_assets: Res<Assets<AsepriteAsset>>,
) {
    commands.spawn(Camera2d);

    let ase_handle = &ase_handles[0];
    let ase_asset = ase_assets.get(ase_handle).unwrap();
    let anim = AsepriteAnimation::new(ase_asset.info(), sprites::Player::tags::STAND);
    commands.spawn((
        Player,
        PlayerState::Stand,
        Orientation::Right,
        Transform {
            scale: Vec3::splat(2.0),
            ..default()
        },
        Sprite {
            image: ase_asset.texture().clone_weak(),
            texture_atlas: Some(TextureAtlas {
                index: anim.current_frame(),
                layout: ase_asset.layout().clone_weak(),
            }),
            ..default()
        },
        Aseprite {
            anim,
            asset: ase_handle.clone_weak(),
        },
    ));
}

fn update_player(
    time: Res<Time>,
    aseprites: Res<Assets<AsepriteAsset>>,
    mut commands: Commands,
    mut ev_player_changed: EventReader<PlayerChanged>,
    mut player_q: Query<
        (
            Entity,
            &mut PlayerState,
            Option<&Movements>,
            &mut Transform,
            &mut Sprite,
            &mut Aseprite,
            &mut Orientation,
        ),
        With<Player>,
    >,
) {
    let (
        player,
        mut player_state,
        movements,
        mut transform,
        mut sprite,
        mut aseprite,
        mut orientation,
    ) = player_q.single_mut();

    for PlayerChanged {
        new_state,
        new_orientation,
        new_movements,
    } in ev_player_changed.read()
    {
        if let Some(new_state) = new_state {
            let info = aseprites.get(&aseprite.asset).unwrap().info();
            aseprite.anim = AsepriteAnimation::new(info, new_state.animation_tag());
            match new_state {
                PlayerState::Stand | PlayerState::Attack => {
                    commands.entity(player).remove::<Movements>();
                }
                _ => (),
            }
            *player_state = *new_state;
        }

        if let Some(new_orientation) = new_orientation {
            sprite.flip_x = new_orientation.flip_x();
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
                Moving::Left => transform.translation.x -= velocity * time.delta_secs(),
                Moving::Right => transform.translation.x += velocity * time.delta_secs(),
                Moving::Up => transform.translation.y += velocity * time.delta_secs(),
                Moving::Down => transform.translation.y -= velocity * time.delta_secs(),
            }
        }
    }
}

fn transition_player(
    time: Res<Time>,
    player_q: Query<(&PlayerState, &Aseprite), With<Player>>,
    aseprites: Res<Assets<AsepriteAsset>>,
    mut ev_player_changed: EventWriter<PlayerChanged>,
) {
    let (&player_state, ase) = player_q.single();
    let ase_asset = aseprites.get(&ase.asset).unwrap();
    if let PlayerState::Attack = player_state {
        let remaining_frames = ase.anim.remaining_tag_frames(ase_asset.info()).unwrap();
        let frame_finished = ase.anim.frame_finished(time.delta());
        if remaining_frames == 0 && frame_finished {
            ev_player_changed.send(PlayerState::Stand.into());
        }
    }
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<&PlayerState, With<Player>>,
    mut ev_player_changed: EventWriter<PlayerChanged>,
) {
    let player_state = player_q.single();

    if keyboard_direction_pressed(&keys) && !keyboard_attack_detected(&keys) {
        match *player_state {
            PlayerState::Attack => {}
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
        ev_player_changed.send(PlayerState::Stand.into());
    } else if keyboard_attack_detected(&keys) {
        if let PlayerState::Attack = *player_state {
            return;
        }
        ev_player_changed.send(PlayerState::Attack.into());
    }
}

fn keyboard_direction_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ArrowLeft)
        || keys.pressed(KeyCode::ArrowUp)
        || keys.pressed(KeyCode::ArrowDown)
        || keys.pressed(KeyCode::ArrowRight)
}

fn keyboard_direction_just_released(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_released(KeyCode::ArrowLeft)
        || keys.just_released(KeyCode::ArrowUp)
        || keys.just_released(KeyCode::ArrowDown)
        || keys.just_released(KeyCode::ArrowRight)
}

fn keyboard_attack_detected(keys: &ButtonInput<KeyCode>) -> bool {
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

#[derive(Default, Event)]
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

impl From<PlayerState> for PlayerChanged {
    fn from(value: PlayerState) -> Self {
        Self::default().new_state(value)
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
    pub fn from_keyboard(keys: &ButtonInput<KeyCode>) -> Self {
        let mut movements = HashSet::with_capacity(4);
        if keys.pressed(KeyCode::ArrowLeft) {
            movements.insert(Moving::Left);
        }
        if keys.pressed(KeyCode::ArrowUp) {
            movements.insert(Moving::Up);
        }
        if keys.pressed(KeyCode::ArrowDown) {
            movements.insert(Moving::Down);
        }
        if keys.pressed(KeyCode::ArrowRight) {
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
