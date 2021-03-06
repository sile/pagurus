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
            data: vec![Rgb::BLACK; size.len()],
            size,
        }
    }

    pub fn resize(&mut self, size: Size) {
        if size != self.size {
            self.size = size;
            self.data = vec![Rgb::BLACK; size.len()];
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn data(&self) -> &[Rgb] {
        &self.data
    }

    pub fn view(&mut self, region: Region) -> Result<CanvasView> {
        self.size
            .to_region()
            .contains(&region)
            .or_fail_with_reason(|_| {
                format!(
                    "failed to create canvas view: canvas_size={:?}, view_region={:?}",
                    self.size, region
                )
            })?;
        Ok(CanvasView {
            canvas: self,
            region,
        })
    }

    pub fn to_view(&mut self) -> CanvasView {
        let region = self.size.to_region();
        CanvasView {
            canvas: self,
            region,
        }
    }

    pub fn draw_sprite(&mut self, offset: Position, sprite: &Sprite) {
        self.to_view().draw_sprite(offset, sprite);
    }

    pub fn fill_rgba(&mut self, color: Rgba) {
        self.to_view().fill_rgba(color);
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

#[derive(Debug)]
pub struct CanvasView<'a> {
    canvas: &'a mut Canvas,
    region: Region,
}

impl<'a> CanvasView<'a> {
    pub fn draw_sprite(&mut self, offset: Position, sprite: &Sprite) {
        let w = self.canvas.size.width as i32;
        for (pixel_pos, pixel) in sprite.pixels() {
            let canvas_pos = pixel_pos + offset + self.region.position;
            if self.region.contains(&canvas_pos) {
                let i = (canvas_pos.y * w + canvas_pos.x) as usize;
                self.canvas.data[i] = pixel.to_alpha_blend_rgb(self.canvas.data[i]);
            }
        }
    }

    pub fn fill_rgba(&mut self, color: Rgba) {
        let w = self.canvas.size.width as i32;
        for pos in self.region.iter() {
            let i = (pos.y * w + pos.x) as usize;
            self.canvas.data[i] = color.to_alpha_blend_rgb(self.canvas.data[i]);
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
            "Sprite {{ image_size: {:?}, sprite_region: {:?}, .. }}",
            self.image_size, self.sprite_region
        )
    }
}
