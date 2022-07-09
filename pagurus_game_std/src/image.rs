use crate::color::{Rgb, Rgba};
use pagurus::failure::OrFail;
use pagurus::spatial::{Contains, Position, Region, Size};
use pagurus::{Result, VideoFrame};
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
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

    pub fn render_sprite(&mut self, offset: Position, sprite: &Sprite) {
        let canvas_region = self.size.to_region();
        for (pixel_pos, pixel) in sprite.pixels() {
            let canvas_pos = pixel_pos + offset;
            if canvas_region.contains(&canvas_pos) {
                let i = canvas_pos.y as usize * self.size.width as usize + canvas_pos.x as usize;
                self.data[i] = pixel.to_alpha_blend_rgb(self.data[i]);
            }
        }
    }

    pub fn to_video_frame(&self) -> Result<VideoFrame<Vec<u8>>> {
        let mut bytes = Vec::with_capacity(self.size.len() * 3);
        for pixel in &self.data {
            bytes.push(pixel.r);
            bytes.push(pixel.g);
            bytes.push(pixel.b);
        }
        VideoFrame::new(bytes, self.size.width).or_fail()
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
            .contains(&region)
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

    pub fn pixels(&self) -> impl '_ + Iterator<Item = (Position, Rgba)> {
        let w = self.image_size.width as usize;
        self.sprite_region.iter().map(move |pos| {
            let Position { x, y } = pos;
            let pixel = self.image_data[y as usize * w + x as usize];
            (pos, pixel)
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
