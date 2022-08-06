use crate::spatial::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PixelFormat {
    Rgb16Big,
    Rgb16Little,
    Rgb24,
    Rgb32,
}

impl PixelFormat {
    fn bytes(self) -> usize {
        match self {
            PixelFormat::Rgb16Big => 2,
            PixelFormat::Rgb16Little => 2,
            PixelFormat::Rgb24 => 3,
            PixelFormat::Rgb32 => 4,
        }
    }
}

#[derive(Debug)]
pub struct VideoFrame {
    format: PixelFormat,
    data: Vec<u8>,
    resolution: Size,
}

impl VideoFrame {
    pub fn new(format: PixelFormat) -> Self {
        Self {
            format,
            data: Vec::new(),
            resolution: Size::EMPTY,
        }
    }

    pub fn format(&self) -> PixelFormat {
        self.format
    }

    pub fn reset_resolution(&mut self, resolution: Size) {
        self.resolution = resolution;
        self.data = vec![0; resolution.len() * self.format.bytes()];
    }

    pub fn resolution(&self) -> Size {
        self.resolution
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}
