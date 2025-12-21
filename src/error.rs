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
