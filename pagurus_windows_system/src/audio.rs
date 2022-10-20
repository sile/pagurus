use pagurus::{failure::OrFail, Result};
use windows::Win32::{
    Media::Audio::XAudio2::{
        self, IXAudio2, XAudio2CreateWithVersionInfo, XAUDIO2_DEFAULT_PROCESSOR,
    },
    System::{
        Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
        SystemInformation::NTDDI_VERSION,
    },
};

#[derive(Debug)]
pub struct AudioPlayer {
    audio: IXAudio2,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let mut audio = None;
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).or_fail()?;
            XAudio2CreateWithVersionInfo(&mut audio, 0, XAUDIO2_DEFAULT_PROCESSOR, NTDDI_VERSION)
                .or_fail()?;
        }
        Ok(Self {
            audio: audio.or_fail()?,
        })
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
