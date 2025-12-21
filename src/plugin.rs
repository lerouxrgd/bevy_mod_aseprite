use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::anim::{AsepriteAnimation, refresh_animations, update_animations};
use crate::info::AsepriteInfo;
use crate::loader::AsepriteLoader;

pub struct AsepritePlugin;

impl Plugin for AsepritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<AsepriteAsset>()
            .register_asset_loader(AsepriteLoader)
            .add_systems(Update, update_animations.in_set(AsepriteSystems::Animate))
            .add_systems(Update, refresh_animations.in_set(AsepriteSystems::Refresh));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AsepriteSystems {
    Animate,
    Refresh,
}

#[derive(Debug, TypePath, Asset)]
pub struct AsepriteAsset {
    /// Info stores data such as tags and slices
    pub info: AsepriteInfo,

    /// TextureAtlasLayout that gets built from the frame info of the Aseprite file
    pub atlas_layout: Handle<TextureAtlasLayout>,

    /// The actual atlas image
    pub atlas_texture: Handle<Image>,
}

/// A component for drawing sprites from an Aseprite animation
#[derive(Component, Default, Clone)]
#[require(Sprite)]
pub struct Aseprite {
    pub asset: Handle<AsepriteAsset>,
    pub anim: AsepriteAnimation,
}
