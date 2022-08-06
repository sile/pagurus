use crate::ogg::AudioDataStream;
use pagurus::{
    audio::AudioData,
    event::{Event, TimeoutEvent},
    failure::OrFail,
    ActionId, Result, System,
};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct AudioPlayer {
    stream: Option<AudioDataStream>,
    ongoing: Option<ActionId>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play<S: System>(&mut self, system: &mut S, stream: AudioDataStream) -> Result<()> {
        if self.stream.is_some() {
            log::warn!("audio mixing is not supported yet (FIXME)");
            return Ok(());
        };

        self.stream = Some(stream);
        self.enqueue_audio_data(system, true).or_fail()?;
        Ok(())
    }

    pub fn handle_event<S: System>(
        &mut self,
        system: &mut S,
        event: Event,
    ) -> Result<Option<Event>> {
        if let Some(ongoing_id) = self.ongoing {
            match event {
                Event::Timeout(TimeoutEvent { id }) if id == ongoing_id => {
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

                let id = if is_first {
                    system.clock_set_timeout(Duration::from_secs(0))
                } else {
                    let duration = samples_duration(enqueued);
                    system.clock_set_timeout(duration)
                };
                self.ongoing = Some(id);
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
