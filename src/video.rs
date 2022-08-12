use crate::failure::{Failure, OrFail};
use crate::spatial::{Position, Size};
use crate::Result;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoFrameSpec {
    pub pixel_format: PixelFormat,
    pub resolution: Size,
    pub stride: u32,
}

impl VideoFrameSpec {
    pub fn data_len(&self) -> usize {
        (self.resolution.height * self.stride) as usize * self.pixel_format.bytes()
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixelFormat {
    Rgb16Be = 0,
    Rgb16Le = 1,
    #[default]
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

#[derive(Debug, Default)]
pub struct VideoFrame<B = Vec<u8>> {
    spec: VideoFrameSpec,
    data: B,
}

impl VideoFrame<Vec<u8>> {
    pub fn new(spec: VideoFrameSpec) -> Self {
        let data = vec![255; spec.resolution.len() * spec.pixel_format.bytes()];
        Self { spec, data }
    }

    pub fn as_ref(&self) -> VideoFrame<&[u8]> {
        VideoFrame {
            spec: self.spec.clone(),
            data: &self.data,
        }
    }

    #[inline]
    pub fn write_rgb(&mut self, pos: Position, r: u8, g: u8, b: u8) {
        let d = &mut self.data;
        let i = pos.y as usize * self.spec.stride as usize + pos.x as usize;
        match self.spec.pixel_format {
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
    pub fn with_data(spec: VideoFrameSpec, data: B) -> Result<Self> {
        (data.as_ref().len() == spec.data_len()).or_fail()?;
        Ok(Self { spec, data })
    }

    pub fn spec(&self) -> &VideoFrameSpec {
        &self.spec
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    // TODO: remove
    pub fn position_to_index(&self, pos: Position) -> usize {
        pos.y as usize * self.spec.stride as usize + pos.x as usize
    }

    #[inline]
    pub fn read_rgb(&self, pos: Position) -> (u8, u8, u8) {
        let d = self.data();
        let i = pos.y as usize * self.spec.stride as usize + pos.x as usize;
        match self.spec.pixel_format {
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
