#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

mod anim;
pub mod info;
mod loader;
mod plugin;

pub use crate::anim::{AsepriteAnimation, AsepriteSlice, AsepriteTag};
pub use crate::info::AsepriteInfo;
pub use crate::loader::{AsepriteLoader, AsepriteLoaderError};
pub use crate::plugin::{Aseprite, AsepriteAsset, AsepritePlugin, AsepriteSystems};

pub mod prelude {
    pub use super::{
        Aseprite, AsepriteAnimation, AsepriteAsset, AsepriteInfo, AsepritePlugin, AsepriteSlice,
        AsepriteSystems, AsepriteTag, aseprite,
    };
}

/// Generates static tags and slices descriptions of an Aserpite animation.
///
/// Calling it as follows:
///
/// ```rust
/// # use bevy_mod_aseprite::aseprite;
/// #[allow(non_snake_case)]
/// #[aseprite(file = "player.ase")]
/// pub mod Player {}
/// ```
///
/// Will generate:
///
/// ```rust
/// #[allow(non_snake_case)]
/// pub mod Player {
///     pub const PATH: &'static str = "player.ase";
///     pub mod tags {
///         pub const WOUND:  ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("wound");
///         pub const STAND:  ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("stand");
///         pub const MOVE:   ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("move");
///         pub const ATTACK: ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("attack");
///         pub const DIE:    ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("die");
///         pub const JUMP:   ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("jump");
///         pub const FALL:   ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("fall");
///         pub const DASH:   ::bevy_mod_aseprite::AsepriteTag = ::bevy_mod_aseprite::AsepriteTag::new("dash");
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
///
///     #[allow(non_snake_case)]
///     #[aseprite(file = "player.ase")]
///     pub mod Player {}
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
pub use bevy_mod_aseprite_derive::aseprite;
