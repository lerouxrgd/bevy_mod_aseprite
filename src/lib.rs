#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

mod anim;
mod error;
mod loader;

use bevy::prelude::*;
use bevy::reflect::TypePath;

pub use self::anim::{AsepriteAnimation, AsepriteTag};
pub use bevy_aseprite_derive::aseprite;
pub use bevy_aseprite_reader::{
    raw::{AsepriteAnimationDirection, AsepriteColor, AsepriteNinePatchInfo},
    AsepriteFrameInfo, AsepriteInfo, AsepritePalette, AsepriteSlice,
};
pub use error::AsepriteLoaderError;

pub struct AsepritePlugin;

impl Plugin for AsepritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<Aseprite>()
            .register_asset_loader(loader::AsepriteLoader)
            .add_systems(
                Update,
                anim::update_animations.in_set(AsepriteSystems::Animate),
            )
            .add_systems(
                Update,
                anim::refresh_animations.in_set(AsepriteSystems::Refresh),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AsepriteSystems {
    Animate,
    Refresh,
}

#[derive(Debug, Clone, TypePath, Asset)]
pub struct Aseprite {
    /// Info stores data such as tags and slices
    info: AsepriteInfo,
    /// TextureAtlasLayout that gets built from the frame info of the Aseprite file
    atlas_layout: Handle<TextureAtlasLayout>,
    ///
    atlas_texture: Handle<Image>,
}

impl Aseprite {
    pub fn info(&self) -> &AsepriteInfo {
        &self.info
    }

    pub fn layout(&self) -> &Handle<TextureAtlasLayout> {
        &self.atlas_layout
    }

    pub fn texture(&self) -> &Handle<Image> {
        &self.atlas_texture
    }
}

/// A Bundle of components for drawing sprites from an Aseprite animation
#[derive(Default, Bundle)]
pub struct AsepriteBundle {
    pub aseprite: Handle<Aseprite>,
    pub animation: AsepriteAnimation,
    pub texture: Handle<Image>,
    pub sprite: Sprite,
    pub atlas: TextureAtlas,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
