use pagurus::{
    audio::{AudioData, AudioSpec, SampleFormat},
    failure::OrFail,
    Result,
};
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
    is_odd: bool,
}

impl AudioPlayer {
    pub fn new(sample_rate: u16, data_samples: usize) -> Result<Self> {
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
                    u32::from(AudioSpec::CHANNELS),
                    u32::from(sample_rate),
                    0,    // Flags,
                    None, // szDeviceId
                    None, // pEffectChain
                    AudioCategory_GameMedia,
                )
                .or_fail()?;
            mastering_voice.or_fail()?
        };

        let format = SampleFormat::I16Le;
        let source_voice = unsafe {
            let mut source_voice = None;
            let n_block_align = format.bytes() as u16;
            let w_bits_per_sample = (n_block_align * 8) * u16::from(AudioSpec::CHANNELS);
            let source_format = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_PCM as u16,
                nChannels: u16::from(AudioSpec::CHANNELS),
                nSamplesPerSec: u32::from(sample_rate),
                nAvgBytesPerSec: u32::from(sample_rate) * u32::from(w_bits_per_sample) / 8,
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
            buf: vec![0; data_samples * 2],
            is_odd: false,
        })
    }

    pub fn play(&mut self, data: AudioData<&[u8]>) -> Result<()> {
        let offset = if self.is_odd {
            self.is_odd = false;
            0
        } else {
            self.is_odd = true;
            data.bytes().len()
        };
        self.buf[offset..].copy_from_slice(data.bytes());

        unsafe {
            let buffer = XAUDIO2_BUFFER {
                Flags: 0,
                AudioBytes: data.bytes().len() as u32,
                pAudioData: self.buf[offset..].as_ptr(),
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
