use crate::failure::{Failure, OrFail};
use crate::spatial::{Position, Size};
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixelFormat {
    Rgb16Be = 0,
    Rgb16Le = 1,
    Rgb24 = 2,
    Rgb32 = 3,
}

impl PixelFormat {
    pub const fn bytes(self) -> usize {
        match self {
            PixelFormat::Rgb16Be => 2,
            PixelFormat::Rgb16Le => 2,
            PixelFormat::Rgb24 => 3,
            PixelFormat::Rgb32 => 4,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(x: u8) -> Result<Self> {
        match x {
            0 => Ok(Self::Rgb16Be),
            1 => Ok(Self::Rgb16Le),
            2 => Ok(Self::Rgb24),
            3 => Ok(Self::Rgb32),
            _ => Err(Failure::new(format!("unknown pixel format: {x}"))),
        }
    }
}

#[derive(Debug)]
pub struct VideoFrame<B = Vec<u8>> {
    format: PixelFormat,
    data: B,
    resolution: Size,
}

impl Default for VideoFrame {
    fn default() -> Self {
        Self::empty(PixelFormat::Rgb24)
    }
}

impl VideoFrame<Vec<u8>> {
    pub fn empty(format: PixelFormat) -> Self {
        Self {
            format,
            data: Vec::new(),
            resolution: Size::EMPTY,
        }
    }

    pub fn set_resolution(&mut self, resolution: Size) {
        self.resolution = resolution;
        self.data = vec![255; resolution.len() * self.format.bytes()];
    }

    pub fn as_ref(&self) -> VideoFrame<&[u8]> {
        VideoFrame {
            format: self.format,
            resolution: self.resolution,
            data: &self.data,
        }
    }

    #[inline]
    pub fn write_rgb(&mut self, pos: Position, r: u8, g: u8, b: u8) {
        let d = &mut self.data;
        let i = pos.y as usize * self.resolution.width as usize + pos.x as usize;
        match self.format {
            PixelFormat::Rgb16Be => {
                let r = u16::from(r);
                let g = u16::from(g);
                let b = u16::from(b);
                let v = ((r << 8) & 0xf800) | ((g << 3) & 0x07e0) | (b >> 3);
                d[i * 2] = (v >> 8) as u8;
                d[i * 2 + 1] = v as u8;
            }
            PixelFormat::Rgb16Le => {
                let r = u16::from(r);
                let g = u16::from(g);
                let b = u16::from(b);
                let v = ((r << 8) & 0xf800) | ((g << 3) & 0x07e0) | (b >> 3);
                d[i * 2] = v as u8;
                d[i * 2 + 1] = (v >> 8) as u8;
            }
            PixelFormat::Rgb24 => {
                d[i * 3] = r;
                d[i * 3 + 1] = g;
                d[i * 3 + 2] = b;
            }
            PixelFormat::Rgb32 => {
                d[i * 4] = r;
                d[i * 4 + 1] = g;
                d[i * 4 + 2] = b;
            }
        }
    }
}

impl<B: AsRef<[u8]>> VideoFrame<B> {
    pub fn new(format: PixelFormat, data: B, resolution: Size) -> Result<Self> {
        (data.as_ref().len() == resolution.len() * format.bytes()).or_fail()?;
        Ok(Self {
            format,
            data,
            resolution,
        })
    }

    pub fn format(&self) -> PixelFormat {
        self.format
    }

    pub fn resolution(&self) -> Size {
        self.resolution
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    #[inline]
    pub fn read_rgb(&self, pos: Position) -> (u8, u8, u8) {
        let d = self.data();
        let i = pos.y as usize * self.resolution.width as usize + pos.x as usize;
        match self.format {
            PixelFormat::Rgb16Be => {
                let v = u16::from(d[i * 2]) << 8 | u16::from(d[i * 2 + 1]);
                (
                    (v >> 11) as u8,
                    (v >> 5) as u8 & 0b111111,
                    v as u8 & 0b11111,
                )
            }
            PixelFormat::Rgb16Le => {
                let v = u16::from(d[i * 2]) | u16::from(d[i * 2 + 1]) << 8;
                (
                    (v >> 11) as u8,
                    (v >> 5) as u8 & 0b111111,
                    v as u8 & 0b11111,
                )
            }
            PixelFormat::Rgb24 => (d[i * 3], d[i * 3 + 1], d[i * 3 + 2]),
            PixelFormat::Rgb32 => (d[i * 4], d[i * 4 + 1], d[i * 4 + 2]),
        }
    }
}
