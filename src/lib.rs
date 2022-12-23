#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

mod anim;
mod loader;

use bevy::prelude::*;
use bevy::reflect::TypeUuid;

pub use self::anim::{AsepriteAnimation, AsepriteTag};
pub use bevy_aseprite_derive::aseprite;
pub use bevy_aseprite_reader::{
    raw::{AsepriteAnimationDirection, AsepriteColor, AsepriteNinePatchInfo},
    AsepriteFrameInfo, AsepriteInfo, AsepritePalette, AsepriteSlice,
};

pub struct AsepritePlugin;

impl Plugin for AsepritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<Aseprite>()
            .add_asset_loader(loader::AsepriteLoader)
            .add_system(loader::process_load)
            .add_system(loader::insert_sprite_sheet)
            .add_system(anim::update_animations.after(loader::insert_sprite_sheet));
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f72b299c-1158-491a-8bfb-53710622bf19"]
pub struct Aseprite {
    /// Info stores data such as tags and slices
    info: AsepriteInfo,
    /// Atlas that gets built from the frame info of the aseprite file
    atlas: loader::AsepriteAtlas,
}

impl Aseprite {
    pub fn info(&self) -> &AsepriteInfo {
        &self.info
    }
}

/// A bundle defining a drawn aseprite
#[derive(Default, Bundle)]
pub struct AsepriteBundle {
    #[bundle]
    pub transform: TransformBundle,
    pub aseprite: Handle<Aseprite>,
    pub animation: AsepriteAnimation,
}
