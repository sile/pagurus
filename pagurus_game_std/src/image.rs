use crate::color::{Rgb, Rgba};
use pagurus::spatial::{Region, Size};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Canvas {
    data: Vec<Rgb>,
    size: Size,
}

impl Canvas {
    pub fn new(size: Size) -> Self {
        Self {
            data: size.iter().map(|_| Rgb::WHITE).collect(),
            size,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn data(&self) -> &[Rgb] {
        &self.data
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    data: Arc<Vec<Rgba>>,
    region: Region,
}

impl Sprite {
    pub fn new(data: Vec<Rgba>, size: Size) -> Result<Self, SpriteError> {
        if data.len() != size.len() {
            return Err(SpriteError::SizeMismatch {
                pixels: data.len(),
                size,
            });
        }
        Ok(Self {
            data: Arc::new(data),
            region: Region::from(size),
        })
    }

    pub fn data(&self) -> &[Rgba] {
        &self.data
    }

    pub fn region(&self) -> Region {
        self.region
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpriteError {
    #[error("expected {size:?}, but got {pixels} pixels image data")]
    SizeMismatch { pixels: usize, size: Size },
}
