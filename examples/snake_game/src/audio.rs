use ffmml::{Music, MusicPlayer};
use pagurus::{
    audio::{AudioData, Sample},
    event::{Event, TimeoutEvent, TimeoutTag},
    System,
};
use std::time::Duration;

const SAMPLE_RATE: u16 = 48000;
const DATA_SAMPLES: usize = 960; // 20ms
const TIMEOUT_TAG: TimeoutTag = TimeoutTag::new(1);

#[derive(Debug, Default)]
pub struct AudioMixer {
    start_time: Duration,
    total_samples: u64,
    audio_data: AudioData,
    musics: Vec<MusicPlayer>,
}

impl AudioMixer {
    pub fn init<S: System>(&mut self, system: &mut S) {
        self.start_time = system.clock_game_time();
        self.audio_data = AudioData::new(system.audio_init(SAMPLE_RATE, DATA_SAMPLES));
        system.clock_set_timeout(TIMEOUT_TAG, Duration::from_secs(0));
    }

    pub fn play_click_sound(&mut self) {
        let mml = "@v0 = { 10 9 8 7 6 5 4 3 2 1 0 } A @01 @v0 f";
        self.play(&mml.parse().expect("unreachable"));
    }

    pub fn play_crash_sound(&mut self) {
        let mml = "@v0 = { 14 15 15 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0 } D @v0 g";
        self.play(&mml.parse().expect("unreachable"));
    }

    pub fn play_eat_sound(&mut self) {
        let mml = "@v0 = { 10 9 8 7 6 5 4 3 2 1 0 } A @02 @v0 c";
        self.play(&mml.parse().expect("unreachable"));
    }

    fn play(&mut self, music: &Music) {
        self.musics.push(music.play(SAMPLE_RATE));
    }

    pub fn handle_event<S: System>(&mut self, system: &mut S, event: &Event) {
        if !matches!(
            event,
            Event::Timeout(TimeoutEvent {
                tag: TIMEOUT_TAG,
                ..
            })
        ) {
            return;
        }

        if self.musics.is_empty() {
            self.total_samples += DATA_SAMPLES as u64;
        } else {
            for i in 0..DATA_SAMPLES {
                let sample = self.sample();
                self.audio_data.write_sample(i, sample);
            }
            system.audio_enqueue(self.audio_data.as_ref());
        }

        let elapsed = Duration::from_secs(self.total_samples) / u32::from(SAMPLE_RATE);
        let wait = elapsed.saturating_sub(system.clock_game_time());
        system.clock_set_timeout(TIMEOUT_TAG, wait);
    }

    fn sample(&mut self) -> Sample {
        let mut sample = ffmml::Sample::ZERO;
        let mut i = 0;
        while i < self.musics.len() {
            if let Some(x) = self.musics[i].next() {
                sample = sample + x;
                i += 1;
            } else {
                self.musics.swap_remove(i);
            }
        }
        self.total_samples += 1;
        Sample::F32(sample.get())
    }
}
