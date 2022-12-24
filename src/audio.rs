use crate::Result;
use orfail::Failure;

#[derive(Debug, Default)]
pub struct AudioData<B = Vec<u8>> {
    spec: AudioSpec,
    data: B,
}

impl AudioData<Vec<u8>> {
    pub fn new(spec: AudioSpec) -> Self {
        Self {
            spec,
            data: vec![0; spec.data_samples * spec.sample_format.bytes()],
        }
    }

    #[inline]
    pub fn write_sample(&mut self, i: usize, sample: impl Into<Sample>) {
        let sample = sample.into();
        match self.spec.sample_format {
            SampleFormat::I16Be => {
                self.data[i * 2..][..2].copy_from_slice(&sample.to_i16().to_be_bytes());
            }
            SampleFormat::I16Le => {
                self.data[i * 2..][..2].copy_from_slice(&sample.to_i16().to_le_bytes());
            }
            SampleFormat::F32Be => {
                self.data[i * 4..][..4].copy_from_slice(&sample.to_f32().to_be_bytes());
            }
            SampleFormat::F32Le => {
                self.data[i * 4..][..4].copy_from_slice(&sample.to_f32().to_le_bytes());
            }
        }
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }

    pub fn as_ref(&self) -> AudioData<&[u8]> {
        AudioData {
            spec: self.spec,
            data: &self.data,
        }
    }
}

impl<B: AsRef<[u8]>> AudioData<B> {
    pub fn spec(&self) -> AudioSpec {
        self.spec
    }

    pub fn samples(&self) -> Samples {
        Samples {
            spec: self.spec,
            data: self.data.as_ref(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.data.as_ref()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "wasm",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AudioSpec {
    pub sample_format: SampleFormat,
    pub sample_rate: u16,
    pub data_samples: usize,
}

impl AudioSpec {
    pub const CHANNELS: u8 = 1;
}

#[derive(Debug)]
pub struct Samples<'a> {
    spec: AudioSpec,
    data: &'a [u8],
}

impl<'a> Samples<'a> {
    pub fn len(&self) -> usize {
        self.spec.data_samples
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> Iterator for Samples<'a> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        match self.spec.sample_format {
            SampleFormat::I16Be => {
                let v = i16::from_be_bytes([self.data[0], self.data[1]]);
                self.data = &self.data[2..];
                Some(Sample::I16(v))
            }
            SampleFormat::I16Le => {
                let v = i16::from_le_bytes([self.data[0], self.data[1]]);
                self.data = &self.data[2..];
                Some(Sample::I16(v))
            }
            SampleFormat::F32Be => {
                let v =
                    f32::from_be_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
                self.data = &self.data[4..];
                Some(Sample::F32(v))
            }
            SampleFormat::F32Le => {
                let v =
                    f32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
                self.data = &self.data[4..];
                Some(Sample::F32(v))
            }
        }
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "wasm",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "UPPERCASE")
)]
pub enum SampleFormat {
    #[default]
    I16Be = 0,
    I16Le = 1,
    F32Be = 2,
    F32Le = 3,
}

impl SampleFormat {
    pub const fn bytes(self) -> usize {
        match self {
            SampleFormat::I16Be => 2,
            SampleFormat::I16Le => 2,
            SampleFormat::F32Be => 4,
            SampleFormat::F32Le => 4,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(x: u8) -> Result<Self> {
        match x {
            0 => Ok(Self::I16Be),
            1 => Ok(Self::I16Le),
            2 => Ok(Self::F32Be),
            3 => Ok(Self::F32Le),
            _ => Err(Failure::new().message(format!("unknown audio sample format: {x}"))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Sample {
    I16(i16),
    F32(f32),
}

impl Sample {
    #[inline]
    pub fn to_i16(self) -> i16 {
        match self {
            Sample::I16(v) => v,
            Sample::F32(v) => {
                let v = v.clamp(-1.0, 1.0);
                if v < 0.0 {
                    (-v * f32::from(i16::MIN)) as i16
                } else {
                    (v * f32::from(i16::MAX)) as i16
                }
            }
        }
    }

    #[inline]
    pub fn to_f32(self) -> f32 {
        match self {
            Sample::F32(v) => v,
            Sample::I16(v) => {
                if v < 0 {
                    -f32::from(v) / f32::from(i16::MIN)
                } else {
                    f32::from(v) / f32::from(i16::MAX)
                }
            }
        }
    }
}

impl From<i16> for Sample {
    fn from(value: i16) -> Self {
        Self::I16(value)
    }
}

impl From<f32> for Sample {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}
