use crate::color::{Rgb, Rgba};
use pagurus::failure::OrFail;
use pagurus::spatial::{Region, Size};
use pagurus::Result;
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

#[derive(Clone)]
pub struct Sprite {
    image_data: Arc<Vec<Rgba>>,
    image_size: Size,
    sprite_region: Region,
}

impl Sprite {
    pub fn from_rgb24_bytes(bytes: &[u8], size: Size) -> Result<Self> {
        (bytes.len() % 3 == 0).or_fail()?;
        (bytes.len() / 3 == size.len()).or_fail()?;

        Ok(Self {
            image_data: Arc::new(
                bytes
                    .chunks(3)
                    .map(|x| Rgba::new(x[0], x[1], x[2], 255))
                    .collect(),
            ),
            image_size: size,
            sprite_region: size.into(),
        })
    }

    pub fn from_rgba32_bytes(bytes: &[u8], size: Size) -> Result<Self> {
        (bytes.len() % 4 == 0).or_fail()?;
        (bytes.len() / 4 == size.len()).or_fail()?;

        Ok(Self {
            image_data: Arc::new(
                bytes
                    .chunks(4)
                    .map(|x| Rgba::new(x[0], x[1], x[2], x[3]))
                    .collect(),
            ),
            image_size: size,
            sprite_region: size.into(),
        })
    }

    pub fn original(&self) -> Self {
        Self {
            image_data: Arc::clone(&self.image_data),
            image_size: self.image_size,
            sprite_region: self.image_size.into(),
        }
    }

    pub fn size(&self) -> Size {
        self.sprite_region.size
    }

    pub fn clip(&self, region: Region) -> Result<Self> {
        Region::from(self.size())
            .contains(region)
            .or_fail_with_reason(|_| {
                format!(
                    "failed to clip a sprite: clip_region={:?}, sprite_size={:?}",
                    region,
                    self.size()
                )
            })?;
        Ok(Self {
            image_data: Arc::clone(&self.image_data),
            image_size: self.image_size,
            sprite_region: Region::new(self.sprite_region.position + region.position, region.size),
        })
    }
}

impl std::fmt::Debug for Sprite {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Sprite {{ image_size: {:?}, sprite_region: {:?}, .. }}",
            self.image_size, self.sprite_region
        )
    }
}
