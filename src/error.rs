use bevy_aseprite_reader as reader;

#[derive(Debug)]
pub enum AsepriteLoaderError {
    Aseprite(reader::error::AsepriteError),
    Io(std::io::Error),
    TextureAccess(bevy::image::TextureAccessError),
}

impl From<reader::error::AsepriteError> for AsepriteLoaderError {
    fn from(value: reader::error::AsepriteError) -> Self {
        Self::Aseprite(value)
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
