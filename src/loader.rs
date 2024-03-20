use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::{BevyDefault, TextureFormatPixelInfo},
    },
};

use crate::error::AsepriteLoaderError;
use crate::{Aseprite, AsepriteInfo};

#[derive(Debug, Default)]
pub struct AsepriteLoader;

impl AssetLoader for AsepriteLoader {
    type Asset = Aseprite;
    type Settings = ();
    type Error = AsepriteLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            debug!("Loading aseprite at {:?}", load_context.path());

            let mut buffer = vec![];
            reader.read_to_end(&mut buffer).await?;
            let ase_data = bevy_aseprite_reader::Aseprite::from_bytes(buffer)?;

            let frames = ase_data.frames();
            let ase_images = frames
                .get_for(&(0..frames.count() as u16))
                .get_images()
                .unwrap();

            let rects: Vec<Rect> = ase_images
                .iter()
                .enumerate()
                .map(|(i, texture)| {
                    let min_x = i as u32 * texture.width();
                    let min_y = 0;
                    let max_x = (i + 1) as u32 * texture.width();
                    let max_y = texture.height();
                    Rect::new(min_x as f32, min_y as f32, max_x as f32, max_y as f32)
                })
                .collect();

            let format = TextureFormat::bevy_default();
            let textures = ase_images
                .into_iter()
                .map(|texture| {
                    Image::new(
                        Extent3d {
                            width: texture.width(),
                            height: texture.height(),
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        texture.into_raw(),
                        format,
                        RenderAssetUsages::default(),
                    )
                })
                .collect::<Vec<_>>();

            let info: AsepriteInfo = ase_data.into();

            let (frame_width, frame_height) = info.dimensions;
            let atlas_width = frame_width as u32 * info.frame_count as u32;
            let atlas_height = frame_height as u32;

            let mut atlas_texture = Image::new(
                Extent3d {
                    width: atlas_width,
                    height: atlas_height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![0; format.pixel_size() * (atlas_width * atlas_height) as usize],
                format,
                RenderAssetUsages::default(),
            );
            for (i, texture) in textures.into_iter().enumerate() {
                let rect_width = frame_width as usize;
                let rect_height = frame_height as usize;
                let rect_x = i * frame_width as usize;
                let rect_y = 0;
                let atlas_width = atlas_width as usize;
                let format_size = format.pixel_size();

                for (texture_y, bound_y) in (rect_y..rect_y + rect_height).enumerate() {
                    let begin = (bound_y * atlas_width + rect_x) * format_size;
                    let end = begin + rect_width * format_size;
                    let texture_begin = texture_y * rect_width * format_size;
                    let texture_end = texture_begin + rect_width * format_size;
                    atlas_texture.data[begin..end]
                        .copy_from_slice(&texture.data[texture_begin..texture_end]);
                }
            }
            let atlas_texture = load_context.add_labeled_asset("image".into(), atlas_texture);

            let mut atlas_layout =
                TextureAtlasLayout::new_empty(Vec2::new(atlas_width as f32, atlas_height as f32));
            atlas_layout.textures = rects;
            let atlas_layout = load_context.add_labeled_asset("atlas".into(), atlas_layout);

            Ok(Aseprite {
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
