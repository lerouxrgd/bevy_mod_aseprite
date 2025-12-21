use bevy::asset::uuid::Uuid;
use bevy::asset::{AssetLoader, RenderAssetUsages};
use bevy::log;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::ConditionalSendFuture;

use crate::info::{AsepriteInfo, Palette};
use crate::plugin::AsepriteAsset;

#[derive(Debug, Default)]
pub struct AsepriteLoader;

impl AssetLoader for AsepriteLoader {
    type Asset = AsepriteAsset;
    type Settings = ();
    type Error = AsepriteLoaderError;

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            log::debug!("Loading aseprite at {:?}", load_context.path());

            let mut bytes = vec![];
            reader.read_to_end(&mut bytes).await?;
            let raw = aseprite_loader::loader::AsepriteFile::load(&bytes)?;

            let mut images = Vec::new();
            for (index, _frame) in raw.frames().iter().enumerate() {
                let (width, height) = raw.size();
                let mut buffer = vec![0; width as usize * height as usize * 4];
                let _hash = raw.combined_frame_image(index, buffer.as_mut_slice())?;
                let image = Image::new_fill(
                    Extent3d {
                        width: width as u32,
                        height: height as u32,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    &buffer,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                images.push(image);
            }

            // Atlas

            let mut atlas_builder = TextureAtlasBuilder::default();
            let mut frame_images = Vec::new();
            for image in images.iter() {
                let handle_id = AssetId::Uuid {
                    uuid: Uuid::new_v4(),
                };
                frame_images.push(handle_id);
                atlas_builder.add_texture(Some(handle_id), image);
            }
            let (layout, _source, image) = atlas_builder.build()?;
            let atlas_layout = load_context.add_labeled_asset("atlas_layout".into(), layout);
            let atlas_texture = load_context.add_labeled_asset("atlas_texture".into(), image);

            // Information

            let dimensions = raw.size();

            let mut tags = HashMap::new();
            raw.tags().iter().for_each(|tag| {
                tags.insert(tag.name.clone(), tag.clone());
            });

            let mut slices = HashMap::new();
            raw.slices().iter().for_each(|slice| {
                slices.insert(slice.name.to_string(), slice.slice_keys.clone());
            });

            let frame_count = raw.frames().iter().count();

            let palette = raw
                .file
                .palette
                .as_ref()
                .map(|p| Palette { colors: p.colors });

            let transparent_palette = raw.file.header.transparent_index;

            let frame_durations = raw
                .frames()
                .iter()
                .map(|frame| frame.duration)
                .collect::<Vec<_>>();

            let info = AsepriteInfo {
                dimensions,
                tags,
                slices,
                frame_count,
                palette,
                transparent_palette,
                frame_durations,
            };

            Ok(AsepriteAsset {
                info,
                atlas_texture,
                atlas_layout,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ase", "aseprite"]
    }
}

#[derive(Debug)]
pub enum AsepriteLoaderError {
    LoadSprite(aseprite_loader::loader::LoadSpriteError),
    LoadImage(aseprite_loader::loader::LoadImageError),
    AtlasBuilder(bevy::image::TextureAtlasBuilderError),
    Io(std::io::Error),
    TextureAccess(bevy::image::TextureAccessError),
}

impl From<aseprite_loader::loader::LoadSpriteError> for AsepriteLoaderError {
    fn from(value: aseprite_loader::loader::LoadSpriteError) -> Self {
        Self::LoadSprite(value)
    }
}

impl From<aseprite_loader::loader::LoadImageError> for AsepriteLoaderError {
    fn from(value: aseprite_loader::loader::LoadImageError) -> Self {
        Self::LoadImage(value)
    }
}

impl From<bevy::image::TextureAtlasBuilderError> for AsepriteLoaderError {
    fn from(value: bevy::image::TextureAtlasBuilderError) -> Self {
        Self::AtlasBuilder(value)
    }
}

impl From<std::io::Error> for AsepriteLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<bevy::image::TextureAccessError> for AsepriteLoaderError {
    fn from(value: bevy::image::TextureAccessError) -> Self {
        Self::TextureAccess(value)
    }
}

impl std::fmt::Display for AsepriteLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AsepriteLoaderError {}
