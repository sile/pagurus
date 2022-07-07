use pagurus::Result;
use pagurus::{failure::OrFail, AudioData};
use std::borrow::Cow;
use std::io::Cursor;

type OggStreamReader = lewton::inside_ogg::OggStreamReader<Cursor<Cow<'static, [u8]>>>;

const FRAME_SIZE: usize = AudioData::SAMPLE_RATE as usize / 10 * 2; // 100ms

pub struct AudioDataStream {
    inner: OggStreamReader,
    buf: Vec<u8>,
    eos: bool,
}

impl AudioDataStream {
    pub fn new<T>(ogg: T) -> Result<Self>
    where
        Cow<'static, [u8]>: From<T>,
    {
        let inner = OggStreamReader::new(Cursor::new(ogg.into())).or_fail()?;

        let channels = inner.ident_hdr.audio_channels;
        let sample_rate = inner.ident_hdr.audio_sample_rate;
        (channels == AudioData::CHANNELS).or_fail_with_reason(|_| {
            format!("only monaural audio is supported: channels={channels}")
        })?;
        (sample_rate == AudioData::SAMPLE_RATE).or_fail_with_reason(|_| {
            format!("only 48KHz audio is supported: sample_rate={sample_rate}")
        })?;

        Ok(Self {
            inner,
            buf: Vec::new(),
            eos: false,
        })
    }

    pub fn peek(&mut self) -> Result<Option<AudioData>> {
        self.fill_buf().or_fail()?;
        if self.buf.is_empty() {
            return Ok(None);
        }

        let data = AudioData::new(&self.buf).or_fail()?;
        Ok(Some(data))
    }

    pub fn consume(&mut self, samples: usize) -> Result<()> {
        (samples <= self.buf.len() / 2).or_fail()?;
        self.buf.drain(0..samples * 2); // FIXME: use ring buffer
        Ok(())
    }

    fn fill_buf(&mut self) -> Result<()> {
        if self.buf.len() < FRAME_SIZE && !self.eos {
            while let Some(samples) = self.inner.read_dec_packet_itl().or_fail()? {
                for sample in samples {
                    self.buf.push((sample >> 8) as u8);
                    self.buf.push(sample as u8);
                }

                if self.buf.len() >= FRAME_SIZE {
                    return Ok(());
                }
            }
            self.eos = true;
        }
        Ok(())
    }
}

impl std::fmt::Debug for AudioDataStream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AudioStreamReader {{ .. }}")
    }
}
