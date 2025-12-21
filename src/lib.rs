#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

mod anim;
pub mod info;
mod loader;
mod plugin;

pub use crate::anim::{AsepriteAnimation, AsepriteTag};
pub use crate::info::AsepriteInfo;
pub use crate::loader::{AsepriteLoader, AsepriteLoaderError};
pub use crate::plugin::{Aseprite, AsepriteAsset, AsepritePlugin, AsepriteSystems};

pub mod prelude {
    pub use super::{
        Aseprite, AsepriteAnimation, AsepriteAsset, AsepriteInfo, AsepritePlugin, AsepriteSystems,
        AsepriteTag, aseprite,
    };
}

/// Generates static tags and slices descriptions of an Aserpite animation.
///
/// Calling it as follows:
///
/// ```rust
/// # use bevy_mod_aseprite::aseprite;
/// aseprite!(pub Player, "player.ase");
/// ```
///
/// Will generate:
///
/// ```rust
/// #[allow(non_snake_case)]
/// pub mod Player {
///     pub const PATH: &'static str = "player.ase";
///     pub mod tags {
///         pub const STAND: &'static str = "stand";
///         pub const JUMP: &'static str = "jump";
///         pub const DASH: &'static str = "dash";
///         pub const WOUND: &'static str = "wound";
///         pub const FALL: &'static str = "fall";
///         pub const MOVE: &'static str = "move";
///         pub const DIE: &'static str = "die";
///         pub const ATTACK: &'static str = "attack";
///     }
///     pub mod slices {}
/// }
/// ```
///
/// In bevy code it helps ensuring tags are up to date, as in this example:
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_mod_aseprite::prelude::*;
/// pub mod sprites {
///     use bevy_mod_aseprite::aseprite;
///     aseprite!(pub Player, "player.ase");
/// }
///
/// fn load_assets(asset_server: Res<AssetServer>, mut ase_handles: ResMut<AsepriteHandles>) {
///     let player = asset_server.load(sprites::Player::PATH);
///     ase_handles.push(player);
/// }
///
/// fn setup(
///     mut commands: Commands,
///     ase_handles: Res<AsepriteHandles>,
///     ase_assets: Res<Assets<AsepriteAsset>>,
/// ) {
///     let ase_handle = &ase_handles[0];
///     let ase_asset = ase_assets.get(ase_handle).unwrap();
///     let anim = AsepriteAnimation::new(&ase_asset.info, sprites::Player::tags::STAND);
///
///     // commands.spawn(...);
/// }
///
/// #[derive(Resource, Deref, DerefMut, Default)]
/// struct AsepriteHandles(Vec<Handle<AsepriteAsset>>);
/// ```
pub use bevy_aseprite_derive::aseprite;
