#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

mod anim;
mod error;
mod loader;

use bevy::prelude::*;
use bevy::reflect::TypePath;

pub use self::anim::{AsepriteAnimation, AsepriteTag};
pub use bevy_aseprite_derive::aseprite;
pub use error::AsepriteLoaderError;

pub mod info {
    use bevy::platform::collections::HashMap;

    pub use aseprite_loader::binary::chunks::slice::{NinePatch, Pivot, SliceKey};
    pub use aseprite_loader::binary::chunks::tags::AnimationDirection;
    pub use aseprite_loader::binary::palette::Palette;
    pub use aseprite_loader::binary::scalars::{Byte, Double, Dword, Long};
    pub use aseprite_loader::loader::Tag;

    #[derive(Debug)]
    pub struct AsepriteInfo {
        pub dimensions: (u16, u16),
        pub tags: HashMap<String, Tag>,
        pub slices: HashMap<String, Vec<SliceKey>>,
        pub frame_count: usize,
        pub palette: Option<Palette>,
        pub transparent_palette: Byte,
        pub frame_durations: Vec<u16>, // In milliseconds
    }
}

pub struct AsepritePlugin;

impl Plugin for AsepritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<AsepriteAsset>()
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

#[derive(Debug, TypePath, Asset)]
pub struct AsepriteAsset {
    /// Info stores data such as tags and slices
    info: crate::info::AsepriteInfo,
    /// TextureAtlasLayout that gets built from the frame info of the Aseprite file
    atlas_layout: Handle<TextureAtlasLayout>,
    /// The actual atlas image
    atlas_texture: Handle<Image>,
}

impl AsepriteAsset {
    pub fn info(&self) -> &crate::info::AsepriteInfo {
        &self.info
    }

    pub fn layout(&self) -> &Handle<TextureAtlasLayout> {
        &self.atlas_layout
    }

    pub fn texture(&self) -> &Handle<Image> {
        &self.atlas_texture
    }
}

/// A component for drawing sprites from an Aseprite animation
#[derive(Component, Default)]
#[require(Sprite)]
pub struct Aseprite {
    pub asset: Handle<AsepriteAsset>,
    pub anim: AsepriteAnimation,
}
