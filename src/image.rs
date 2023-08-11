use crate::failure::OrFail;
use crate::spatial::{Contains, Position, Region, Size};
use crate::{video::VideoFrame, Result};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Color {
    Rgb(Rgb),
    Rgba(Rgba),
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb(Rgb::new(r, g, b))
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::Rgba(Rgba::new(r, g, b, a))
    }

    pub fn alpha(self, a: u8) -> Self {
        match self {
            Color::Rgb(x) => Self::Rgba(x.alpha(a)),
            Color::Rgba(mut x) => {
                x.a = a;
                Self::Rgba(x)
            }
        }
    }

    pub fn to_rgba(self) -> Rgba {
        match self {
            Color::Rgb(x) => x.alpha(255),
            Color::Rgba(x) => x,
        }
    }
}

impl From<Rgb> for Color {
    fn from(x: Rgb) -> Self {
        Self::Rgb(x)
    }
}

impl From<Rgba> for Color {
    fn from(x: Rgba) -> Self {
        Self::Rgba(x)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "(u8, u8, u8)", into = "(u8, u8, u8)"))]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const RED: Self = Self::new(255, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn alpha(self, a: u8) -> Rgba {
        Rgba {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::new(r, g, b)
    }
}

impl From<Rgb> for (u8, u8, u8) {
    fn from(x: Rgb) -> Self {
        (x.r, x.g, x.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(from = "(u8, u8, u8, u8)", into = "(u8, u8, u8, u8)")
)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_rgb(self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }

    pub fn to_alpha_blend_rgb(self, dst: Rgb) -> Rgb {
        fn blend(s: u8, d: u8, a: u8) -> u8 {
            let v = u16::from(s) * u16::from(a) + u16::from(d) * (255 - u16::from(a));
            (v / 255) as u8
        }

        Rgb {
            r: blend(self.r, dst.r, self.a),
            g: blend(self.g, dst.g, self.a),
            b: blend(self.b, dst.b, self.a),
        }
    }

    pub fn alpha_blend(self, dst: Self) -> Self {
        if dst.a == 0 {
            return self;
        }

        fn blend(s: u32, d: u32, a: u32) -> u32 {
            s + d - (d * a / (0xFF * 0xFF))
        }

        let a = blend(
            u32::from(self.a) * 0xFF,
            u32::from(dst.a) * 0xFF,
            u32::from(self.a) * 0xFF,
        );
        let r = blend(
            u32::from(self.r) * u32::from(self.a),
            u32::from(dst.r) * u32::from(dst.a),
            u32::from(self.a) * 0xFF,
        );
        let g = blend(
            u32::from(self.g) * u32::from(self.a),
            u32::from(dst.g) * u32::from(dst.a),
            u32::from(self.a) * 0xFF,
        );
        let b = blend(
            u32::from(self.b) * u32::from(self.a),
            u32::from(dst.b) * u32::from(dst.a),
            u32::from(self.a) * 0xFF,
        );
        Self {
            r: (r * 0xFF * 0xFF / a / 0xFF) as u8,
            g: (g * 0xFF * 0xFF / a / 0xFF) as u8,
            b: (b * 0xFF * 0xFF / a / 0xFF) as u8,
            a: (a / 0xFF) as u8,
        }
    }
}

impl From<Rgba> for (u8, u8, u8, u8) {
    fn from(x: Rgba) -> Self {
        (x.r, x.g, x.b, x.a)
    }
}

impl From<(u8, u8, u8, u8)> for Rgba {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Self::new(r, g, b, a)
    }
}

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

    pub fn drawing_region(&self) -> Region {
        self.drawing_region
    }

    pub fn origin(&self) -> Position {
        self.origin
    }

    pub fn frame(&self) -> &VideoFrame {
        self.frame
    }

    // TODO: rename
    pub fn mask_region(&mut self, region: Region) -> Canvas {
        let drawing_region = self.drawing_region.intersection(region + self.origin);
        Canvas {
            frame: self.frame,
            origin: self.origin,
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

#[derive(Clone, Default)]
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

    pub fn from_grayscale8_bytes(bytes: &[u8], size: Size) -> Result<Self> {
        (bytes.len() == size.len()).or_fail()?;

        Ok(Self {
            image_data: Arc::new(
                bytes
                    .iter()
                    .copied()
                    .map(|x| Rgba::new(x, x, x, 255))
                    .collect(),
            ),
            image_size: size,
            sprite_region: size.into(),
        })
    }

    pub fn from_grayscale_alpha16_bytes(bytes: &[u8], size: Size) -> Result<Self> {
        (bytes.len() % 2 == 0).or_fail()?;
        (bytes.len() / 2 == size.len()).or_fail()?;

        Ok(Self {
            image_data: Arc::new(
                bytes
                    .chunks(2)
                    .map(|x| Rgba::new(x[0], x[0], x[0], x[1]))
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
            .or_fail()
            .map_err(|e| {
                e.message(format!(
                    "failed to clip a sprite: clip_region={:?}, sprite_size={:?}",
                    region,
                    self.size()
                ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_blend_works() {
        let black = Rgba::new(0, 0, 0, 255);
        assert_eq!(black, black.alpha_blend(black));

        let transparent = Rgba::new(0, 0, 0, 0);
        assert_eq!(transparent, transparent.alpha_blend(transparent));
    }
}
