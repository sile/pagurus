use libpulse_binding as pulseaudio;
use libpulse_simple_binding as pulseaudio_simple;
use orfail::OrFail;
use pagurus::audio::{AudioData, AudioSpec};

pub struct AudioSystem {
    simple: pulseaudio_simple::Simple,
    sample_rate: u16,
}

impl AudioSystem {
    pub fn new(sample_rate: u16) -> orfail::Result<Self> {
        let pulseaudio_spec = pulseaudio::sample::Spec {
            format: pulseaudio::sample::Format::S16be,
            rate: sample_rate as u32,
            channels: AudioSpec::CHANNELS,
        };
        let simple = pulseaudio_simple::Simple::new(
            None,
            "Pagurus",
            pulseaudio::stream::Direction::Playback,
            None,
            "Sound",
            &pulseaudio_spec,
            None,
            None,
        )
        .or_fail()?;
        Ok(Self {
            simple,
            sample_rate,
        })
    }

    pub fn enqueue(&mut self, data: AudioData<&[u8]>) -> orfail::Result<()> {
        self.simple.write(data.bytes()).or_fail()?;
        Ok(())
    }
}

impl std::fmt::Debug for AudioSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioSystem")
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}
