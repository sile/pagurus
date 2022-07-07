use std::time::Duration;

use pagurus::{
    event::{Event, TimeoutEvent},
    failure::OrFail,
    AudioData, Result, System,
};

use crate::ogg::AudioDataStream;

#[derive(Debug)]
pub struct AudioPlayer {
    stream: Option<AudioDataStream>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn play<S: System>(&mut self, system: &mut S, stream: AudioDataStream) -> Result<()> {
        self.stream
            .is_none()
            .or_fail_with_reason(|_| format!("audio mixing is not supported yet (TODO)"))?;
        self.stream = Some(stream);
        self.enqueue_audio_data(system, true).or_fail()?;
        Ok(())
    }

    pub fn handle_event<S: System>(
        &mut self,
        system: &mut S,
        event: Event,
    ) -> Result<Option<Event>> {
        if self.stream.is_some() {
            match event {
                Event::Timeout(TimeoutEvent { tag: 1234 }) => {
                    self.enqueue_audio_data(system, false).or_fail()?;
                    Ok(None)
                }
                event => Ok(Some(event)),
            }
        } else {
            Ok(Some(event))
        }
    }

    fn enqueue_audio_data<S: System>(&mut self, system: &mut S, is_first: bool) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            if let Some(data) = stream.peek()? {
                let enqueued = system.audio_enqueue(data);
                stream.consume(enqueued).or_fail()?;

                let tag = 1234; // TODO
                if is_first {
                    system.clock_set_timeout(Duration::from_secs(0), tag);
                } else {
                    system.clock_set_timeout(samples_duration(enqueued), tag);
                }
            } else {
                self.stream = None;
            }
        }
        Ok(())
    }
}

pub fn samples_duration(samples: usize) -> Duration {
    Duration::from_secs_f64(samples as f64 / AudioData::SAMPLE_RATE as f64)
}
