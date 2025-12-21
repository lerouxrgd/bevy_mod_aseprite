//! Exposes essential information about the parsed aseprite file.
//!
//! The main struct is [`AsepriteInfo`][] and it uses types from [aseprite-loader][] which
//! parses aseprite file based on the offial [Aseprite File Format][ase-specs].
//!
//! Types used in [`AsepriteInfo`][] are re-exported from [aseprite-loader][].
//!
//! [aseprite-loader]: https://crates.io/crates/aseprite-loader
//! [ase-specs]: https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md

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
