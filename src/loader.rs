use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::Aseprite;

#[derive(Debug, Default)]
pub struct AsepriteLoader;

impl AssetLoader for AsepriteLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            debug!("Loading aseprite at {:?}", load_context.path());

            let ase_data = bevy_aseprite_reader::Aseprite::from_bytes(bytes)?;
            let frames = ase_data.frames();
            let ase_images = frames
                .get_for(&(0..frames.count() as u16))
                .get_images()
                .unwrap();

            let textures = ase_images
                .into_iter()
                .map(|image| {
                    Image::new(
                        Extent3d {
                            width: image.width(),
                            height: image.height(),
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        image.into_raw(),
                        TextureFormat::Rgba8UnormSrgb,
                    )
                })
                .collect::<Vec<_>>();

            let aseprite = Aseprite {
                info: ase_data.into(),
                atlas: AsepriteAtlas::Raw { textures },
            };

            load_context.set_default_asset(LoadedAsset::new(aseprite));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ase", "aseprite"]
    }
}

#[derive(Debug, Clone)]
pub enum AsepriteAtlas {
    Raw {
        /// Raw Aseprite textures
        textures: Vec<Image>,
    },
    Loaded {
        /// Atlas that gets built from the frame info of the aseprite file
        atlas: Handle<TextureAtlas>,
        /// TextureAtlasBuilder might shift the index order when building so
        /// we keep a mapping of frame# -> atlas index here
        frame_to_idx: Vec<usize>,
    },
}

impl AsepriteAtlas {
    pub fn handle(&self) -> Option<Handle<TextureAtlas>> {
        match self {
            Self::Loaded { atlas, .. } => Some(atlas.clone()),
            Self::Raw { .. } => None,
        }
    }

    pub fn frame_to_idx(&self, frame: usize) -> Option<usize> {
        match self {
            Self::Loaded { frame_to_idx, .. } => Some(frame_to_idx[frame]),
            Self::Raw { .. } => None,
        }
    }
}

pub(crate) fn process_load(
    assets: Res<AssetServer>,
    mut aseprite_ev: EventReader<AssetEvent<Aseprite>>,
    mut aseprites: ResMut<Assets<Aseprite>>,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    for event in aseprite_ev.iter() {
        match event {
            AssetEvent::Removed { .. } => (),
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                debug!(
                    "Processing handle {handle:?} pointing at {:?}",
                    assets.get_handle_path(handle)
                );

                // Get the created/modified aseprite data
                match aseprites.get(handle) {
                    Some(aseprite) if matches!(aseprite.atlas, AsepriteAtlas::Loaded { .. }) => {
                        continue
                    }
                    Some(_) => (),
                    None => {
                        error!("Handle {handle:?} not found in Assets<Aseprite>");
                        continue;
                    }
                }
                let Some(aseprite) = aseprites.get_mut(handle) else {
                    error!("Handle {handle:?} not found in Assets<Aseprite>");
                    continue;
                };
                let AsepriteAtlas::Raw { textures } = &mut aseprite.atlas else {
                    error!("Aseprite {aseprite:?} atlas is already loaded");
                    continue;
                };

                let mut frame_handles = vec![];
                let mut atlas = TextureAtlasBuilder::default();
                for texture in textures.iter_mut() {
                    let texture_handle = images.add(texture.clone());
                    frame_handles.push(texture_handle.cast_weak());
                    atlas.add_texture(texture_handle, texture);
                }
                let atlas = match atlas.finish(&mut images) {
                    Ok(atlas) => atlas,
                    Err(err) => {
                        error!(
                            "Couldn't build atlas for: {:?} got: {:?}",
                            assets.get_handle_path(handle),
                            err
                        );
                        continue;
                    }
                };

                let mut frame_to_idx = vec![];
                for handle in frame_handles {
                    let atlas_idx = atlas.get_texture_index(&handle).unwrap();
                    frame_to_idx.push(atlas_idx);
                }

                aseprite.atlas = AsepriteAtlas::Loaded {
                    atlas: atlases.add(atlas),
                    frame_to_idx,
                };
            }
        }
    }
}

pub fn insert_sprite_sheet(
    mut commands: Commands,
    aseprites: Res<Assets<Aseprite>>,
    query: Query<
        (Entity, &Transform, &Handle<Aseprite>),
        Or<(Without<TextureAtlasSprite>, Changed<Handle<Aseprite>>)>,
    >,
) {
    for (entity, &transform, handle) in query.iter() {
        let aseprite = match aseprites.get(handle) {
            Some(aseprite) => aseprite,
            None => {
                debug!("Handle {handle:?} not ready yet");
                continue;
            }
        };
        if let Some(atlas) = aseprite.atlas.handle() {
            commands.entity(entity).insert(SpriteSheetBundle {
                texture_atlas: atlas.clone(),
                transform,
                ..default()
            });
        } else {
            debug!("Aseprite {aseprite:?} not ready yet");
        }
    }
}
