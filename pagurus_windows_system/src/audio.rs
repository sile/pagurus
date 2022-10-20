use pagurus::{audio::AudioData, failure::OrFail, Result};
use windows::Win32::{
    Media::Audio::{
        AudioCategory_GameMedia,
        XAudio2::{
            IXAudio2, IXAudio2MasteringVoice, IXAudio2SourceVoice, XAudio2CreateWithVersionInfo,
            XAUDIO2_BUFFER, XAUDIO2_COMMIT_NOW, XAUDIO2_DEFAULT_PROCESSOR, XAUDIO2_VOICE_NOSRC,
        },
        WAVEFORMATEX, WAVE_FORMAT_PCM,
    },
    System::{
        Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
        SystemInformation::NTDDI_VERSION,
    },
};

#[derive(Debug)]
pub struct AudioPlayer {
    #[allow(dead_code)]
    audio: IXAudio2,

    mastering_voice: IXAudio2MasteringVoice,
    source_voice: IXAudio2SourceVoice,
    buf: Vec<u8>,
    buf_start: usize,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let audio = unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).or_fail()?;

            let mut audio = None;
            XAudio2CreateWithVersionInfo(&mut audio, 0, XAUDIO2_DEFAULT_PROCESSOR, NTDDI_VERSION)
                .or_fail()?;
            audio.or_fail()?
        };
        let mastering_voice = unsafe {
            let mut mastering_voice = None;
            audio
                .CreateMasteringVoice(
                    &mut mastering_voice,
                    AudioData::CHANNELS as u32,
                    AudioData::SAMPLE_RATE,
                    0,    // Flags,
                    None, // szDeviceId
                    None, // pEffectChain
                    AudioCategory_GameMedia,
                )
                .or_fail()?;
            mastering_voice.or_fail()?
        };

        let source_voice = unsafe {
            let mut source_voice = None;
            let n_block_align: u16 = u16::from(AudioData::BIT_DEPTH / 8);
            let w_bits_per_sample =
                u16::from(AudioData::BIT_DEPTH) * u16::from(AudioData::CHANNELS);
            let source_format = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_PCM as u16,
                nChannels: AudioData::CHANNELS as u16,
                nSamplesPerSec: AudioData::SAMPLE_RATE,
                nAvgBytesPerSec: AudioData::SAMPLE_RATE * u32::from(w_bits_per_sample) / 8,
                nBlockAlign: n_block_align,
                wBitsPerSample: w_bits_per_sample,
                cbSize: 0,
            };

            // FIXME: Use `pCallback` to handle `OnBufferEvent`
            audio
                .CreateSourceVoice(
                    &mut source_voice,
                    &source_format,
                    XAUDIO2_VOICE_NOSRC, // Flags
                    1.0,                 // MaxFrequencyRatio
                    None,                // pCallback,
                    None,                // pSendList
                    None,                // pEffectChain
                )
                .or_fail()?;
            source_voice.or_fail()?
        };
        unsafe {
            source_voice.Start(0, XAUDIO2_COMMIT_NOW).or_fail()?;
        }
        Ok(Self {
            audio,
            mastering_voice,
            source_voice,
            buf: vec![0; 1024 * 1024],
            buf_start: 0,
        })
    }

    pub fn play(&mut self, data: AudioData) -> Result<()> {
        (data.bytes().len() < self.buf.len()).or_fail()?;

        if self.buf.len() <= self.buf_start + data.bytes().len() {
            self.buf_start = 0;
        }
        let start = self.buf_start;
        for (i, sample) in data.samples().enumerate() {
            self.buf[self.buf_start + i * 2] = sample as u8;
            self.buf[self.buf_start + i * 2 + 1] = (sample >> 8) as u8;
        }
        self.buf_start += data.bytes().len();

        unsafe {
            let buffer = XAUDIO2_BUFFER {
                Flags: 0,
                AudioBytes: data.bytes().len() as u32,
                pAudioData: self.buf[start..].as_ptr(),
                PlayBegin: 0,
                PlayLength: 0,
                LoopBegin: 0,
                LoopLength: 0,
                LoopCount: 0,
                pContext: std::ptr::null_mut(),
            };
            self.source_voice
                .SubmitSourceBuffer(&buffer, None)
                .or_fail()?;
        }
        Ok(())
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        unsafe {
            self.source_voice.DestroyVoice();
            self.mastering_voice.DestroyVoice();
            CoUninitialize();
        }
    }
}
