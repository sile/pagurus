use crate::color::{Color, Rgb, Rgba};
use pagurus::failure::OrFail;
use pagurus::spatial::{Contains, Position, Region, Size};
use pagurus::{video::VideoFrame, Result};
use std::sync::Arc;

#[derive(Debug)]
pub struct Canvas<'a> {
    frame: &'a mut VideoFrame,
    origin: Position,
    drawing_region: Region,
}

impl<'a> Canvas<'a> {
    pub fn new(frame: &'a mut VideoFrame) -> Self {
        let drawing_region = frame.spec().resolution.to_region();
        Self {
            frame,
            origin: Position::ORIGIN,
            drawing_region,
        }
    }

    pub fn subregion(&mut self, region: Region) -> Canvas {
        let drawing_region = self.drawing_region.intersection(region + self.origin);
        Canvas {
            frame: self.frame,
            origin: region.position + self.origin,
            drawing_region,
        }
    }

    pub fn offset(&mut self, offset: Position) -> Canvas {
        Canvas {
            frame: self.frame,
            origin: self.origin + offset,
            drawing_region: self.drawing_region,
        }
    }

    pub fn fill_color(&mut self, color: Color) {
        for pos in self.drawing_region.iter() {
            self.draw_pixel_unchecked(pos, color);
        }
    }

    pub fn draw_pixel(&mut self, pos: Position, color: Color) {
        let frame_pos = pos + self.origin;
        if self.drawing_region.contains(&frame_pos) {
            self.draw_pixel_unchecked(frame_pos, color);
        }
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite) {
        for (pos, pixel) in sprite.pixels() {
            self.draw_pixel(pos, Color::Rgba(pixel));
        }
    }

    fn draw_pixel_unchecked(&mut self, pos: Position, color: Color) {
        match color {
            Color::Rgb(c) => {
                self.frame.write_rgb(pos, c.r, c.g, c.b);
            }
            Color::Rgba(c) => {
                let (r, g, b) = self.frame.read_rgb(pos);
                let c = c.to_alpha_blend_rgb(Rgb::new(r, g, b));
                self.frame.write_rgb(pos, c.r, c.g, c.b);
            }
        }
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
        self.sprite_region
            .iter()
            .zip(self.size().to_region().iter())
            .map(move |(Position { x, y }, pos)| {
                let pixel = self.image_data[y as usize * w + x as usize];
                (pos, pixel)
            })
    }

    pub fn get_pixel(&self, pos: Position) -> Option<Rgba> {
        let y = (pos.y + self.sprite_region.position.y) as isize;
        let x = (pos.x + self.sprite_region.position.x) as isize;
        let i = y * self.image_size.width as isize + x;
        (0 <= i && i < self.image_data.len() as isize).then(|| self.image_data[i as usize])
    }
}

impl std::fmt::Debug for Sprite {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Sprite {{ size: {:?}, region: {:?} }}",
            self.image_size, self.sprite_region
        )
    }
}
