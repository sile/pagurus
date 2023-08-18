use crate::failure::{Failure, OrFail};
use crate::spatial::{Position, Size};
use crate::Result;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "UPPERCASE")
)]
pub enum PixelFormat {
    #[default]
    Rgb24 = 0,
    Rgb32 = 1,
    Bgr24 = 2,
}

impl PixelFormat {
    pub const fn bytes(self) -> usize {
        match self {
            PixelFormat::Rgb24 => 3,
            PixelFormat::Rgb32 => 4,
            PixelFormat::Bgr24 => 3,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(x: u8) -> Result<Self> {
        match x {
            0 => Ok(Self::Rgb24),
            1 => Ok(Self::Rgb32),
            2 => Ok(Self::Bgr24),
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
        let size = (spec.resolution.height * spec.stride) as usize * spec.pixel_format.bytes();
        let data = vec![255; size];
        Self { spec, data }
    }

    pub fn as_ref(&self) -> VideoFrame<&[u8]> {
        VideoFrame {
            spec: self.spec,
            data: &self.data,
        }
    }

    #[inline]
    pub fn write_rgb(&mut self, pos: Position, r: u8, g: u8, b: u8) {
        let d = &mut self.data;
        let i = pos.y as usize * self.spec.stride as usize + pos.x as usize;
        match self.spec.pixel_format {
            PixelFormat::Rgb24 => {
                d[i * 3] = r;
                d[i * 3 + 1] = g;
                d[i * 3 + 2] = b;
            }
            PixelFormat::Bgr24 => {
                d[i * 3] = b;
                d[i * 3 + 1] = g;
                d[i * 3 + 2] = r;
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

    pub fn spec(&self) -> VideoFrameSpec {
        self.spec
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    #[inline]
    pub fn read_rgb(&self, pos: Position) -> (u8, u8, u8) {
        let d = self.data();
        let i = pos.y as usize * self.spec.stride as usize + pos.x as usize;
        match self.spec.pixel_format {
            PixelFormat::Rgb24 => (d[i * 3], d[i * 3 + 1], d[i * 3 + 2]),
            PixelFormat::Bgr24 => (d[i * 3 + 2], d[i * 3 + 1], d[i * 3]),
            PixelFormat::Rgb32 => (d[i * 4], d[i * 4 + 1], d[i * 4 + 2]),
        }
    }
}
