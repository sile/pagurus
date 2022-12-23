use crate::Result;
use orfail::Failure;

#[derive(Debug)]
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
                self.data[i..].copy_from_slice(&sample.to_i16().to_be_bytes());
            }
            SampleFormat::I16Le => {
                self.data[i..].copy_from_slice(&sample.to_i16().to_le_bytes());
            }
            SampleFormat::F32Be => {
                self.data[i..].copy_from_slice(&sample.to_f32().to_be_bytes());
            }
            SampleFormat::F32Le => {
                self.data[i..].copy_from_slice(&sample.to_f32().to_le_bytes());
            }
        }
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
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

    pub fn samples(&self) -> usize {
        self.data().len() / self.spec.sample_format.bytes()
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSpec {
    pub sample_format: SampleFormat,
    pub sample_rate: u16,
    pub data_samples: usize,
}

impl AudioSpec {
    pub const CHANNELS: u8 = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SampleFormat {
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
