use pagurus::event::{Event, ResourceEvent, TimeoutEvent, WindowEvent};
use pagurus::failure::{Failure, OrFail};
use pagurus::resource::ResourceName;
use pagurus::spatial::Size;
use pagurus::{ActionId, AudioData, GameRequirements, Result, System, VideoFrame};
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, EventSubsystem, VideoSubsystem};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::time::{Duration, Instant, UNIX_EPOCH};

// TODO(?): Use a builder instead?
#[derive(Debug, Clone)]
pub struct SdlSystemOptions {
    pub states_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub files_dir: PathBuf,
}

impl SdlSystemOptions {
    pub const DEFAULT_STATES_DIR: &'static str = "states/";
    pub const DEFAULT_ASSETS_DIR: &'static str = "assets/";
    pub const DEFAULT_FILES_DIR: &'static str = "files/";
}

impl Default for SdlSystemOptions {
    fn default() -> Self {
        Self {
            states_dir: PathBuf::from(Self::DEFAULT_STATES_DIR),
            assets_dir: PathBuf::from(Self::DEFAULT_ASSETS_DIR),
            files_dir: PathBuf::from(Self::DEFAULT_FILES_DIR),
        }
    }
}

pub struct SdlSystem {
    sdl_canvas: Canvas<Window>,
    sdl_event: EventSubsystem,
    sdl_event_pump: EventPump,
    sdl_audio_queue: AudioQueue<i16>,
    start: Instant,
    timeout_queue: BinaryHeap<(Reverse<Duration>, ActionId)>,
    next_action_id: ActionId,
    requirements: GameRequirements,
    options: SdlSystemOptions,
}

impl SdlSystem {
    pub const DEFAULT_TITLE: &'static str = "Pagurus";
    pub const DEFAULT_WINDOW_SIZE: Size = Size::from_wh(800, 600);

    pub fn new(requirements: GameRequirements, options: SdlSystemOptions) -> Result<Self> {
        let window_size = requirements
            .logical_window_size
            .unwrap_or(Self::DEFAULT_WINDOW_SIZE);
        Self::with_canvas(
            |sdl_video| {
                let sdl_window = sdl_video
                    .window(Self::DEFAULT_TITLE, window_size.width, window_size.height)
                    .position_centered()
                    .build()
                    .or_fail()?;
                let sdl_canvas = sdl_window.into_canvas().build().or_fail()?;
                Ok(sdl_canvas)
            },
            requirements,
            options,
        )
        .or_fail()
    }

    pub fn with_canvas<F>(
        canvas: F,
        requirements: GameRequirements,
        options: SdlSystemOptions,
    ) -> Result<Self>
    where
        F: FnOnce(VideoSubsystem) -> Result<Canvas<Window>>,
    {
        let sdl_context = sdl2::init().map_err(Failure::new)?;

        // Video
        let sdl_video = sdl_context.video().map_err(Failure::new)?;
        let mut sdl_canvas = canvas(sdl_video).or_fail()?;
        if let Some(size) = requirements.logical_window_size {
            sdl_canvas
                .set_logical_size(size.width, size.height)
                .or_fail()?;
        }

        // Audio
        let sdl_audio = sdl_context.audio().map_err(Failure::new)?;
        let audio_spec = AudioSpecDesired {
            freq: Some(AudioData::SAMPLE_RATE as i32),
            channels: Some(AudioData::CHANNELS),
            samples: None,
        };
        let sdl_audio_queue = sdl_audio
            .open_queue(None, &audio_spec)
            .map_err(Failure::new)?;
        sdl_audio_queue.resume();

        // Event
        let sdl_event = sdl_context.event().map_err(Failure::new)?;
        sdl_event
            .register_custom_event::<Event>()
            .map_err(Failure::new)?;
        let sdl_event_pump = sdl_context.event_pump().map_err(Failure::new)?;

        Ok(Self {
            sdl_canvas,
            sdl_event,
            sdl_event_pump,
            sdl_audio_queue,
            start: Instant::now(),
            timeout_queue: BinaryHeap::new(),
            next_action_id: ActionId::default(),
            requirements,
            options,
        })
    }

    pub fn logical_window_size(&self) -> Size {
        let (width, height) = self.sdl_canvas.logical_size();
        Size { width, height }
    }

    pub fn wait_event(&mut self) -> Event {
        loop {
            let timeout =
                if let Some((Reverse(expiry_time), id)) = self.timeout_queue.peek().copied() {
                    if let Some(timeout) = expiry_time.checked_sub(self.start.elapsed()) {
                        timeout
                    } else {
                        self.timeout_queue.pop();
                        return Event::Timeout(TimeoutEvent { id });
                    }
                } else {
                    Duration::from_secs(1) // Arbitrary large timeout value
                };

            let event = self
                .sdl_event_pump
                .wait_event_timeout(timeout.as_millis() as u32)
                .and_then(crate::event::to_pagurus_event)
                .and_then(|event| self.filter_event(event));
            if let Some(event) = event {
                return event;
            }
        }
    }

    fn filter_event(&self, event: Event) -> Option<Event> {
        if let Some(window_size) = self.requirements.logical_window_size {
            if matches!(event, Event::Window(WindowEvent::Resized { .. })) {
                return None;
            }
            if let Some(pos) = event.position() {
                if !window_size.contains(pos) {
                    return None;
                }
            }
        }
        Some(event)
    }

    fn resolve_resource_path(&self, name: &ResourceName) -> PathBuf {
        match name {
            ResourceName::State(path) => self.options.states_dir.join(path.as_ref()),
            ResourceName::Asset(path) => self.options.assets_dir.join(path.as_ref()),
            ResourceName::File(path) => self.options.files_dir.join(path.as_ref()),
        }
    }
}

impl System for SdlSystem {
    fn video_render(&mut self, frame: VideoFrame) {
        self.sdl_canvas.clear();

        let texture_creator = self.sdl_canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_static(
                Some(PixelFormatEnum::RGB24),
                frame.size().width,
                frame.size().height,
            )
            .unwrap_or_else(|e| panic!("failed to create a texture: {e}"));
        texture
            .update(None, frame.data(), frame.size().width as usize * 3)
            .unwrap_or_else(|e| panic!("failed to update texture: {e}"));

        self.sdl_canvas
            .copy(&texture, None, None)
            .unwrap_or_else(|e| panic!("failed to copy texture to canvas: {e}"));

        self.sdl_canvas.present();
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        let samples = data.samples().collect::<Vec<_>>();
        self.sdl_audio_queue
            .queue_audio(&samples)
            .unwrap_or_else(|e| panic!("failed to queue audio data: {e}"));
        samples.len()
    }

    fn audio_cancel(&mut self) {
        self.sdl_audio_queue.clear();
    }

    fn console_log(&mut self, message: &str) {
        eprintln!("{message}");
    }

    fn clock_game_time(&mut self) -> Duration {
        self.start.elapsed()
    }

    fn clock_unix_time(&mut self) -> Duration {
        UNIX_EPOCH
            .elapsed()
            .unwrap_or_else(|e| panic!("failed to get UNIX timestamp: {e}"))
    }

    fn clock_set_timeout(&mut self, timeout: Duration) -> ActionId {
        let id = self.next_action_id.get_and_increment();
        let time = self.start.elapsed() + timeout;
        self.timeout_queue.push((Reverse(time), id));
        id
    }

    fn resource_put(&mut self, name: &ResourceName, data: &[u8]) -> ActionId {
        let id = self.next_action_id.get_and_increment();
        let path = self.resolve_resource_path(name);
        let event_tx = self.sdl_event.event_sender();
        let data = data.to_owned();
        // TODO: use an I/O thread to serialize request handling
        std::thread::spawn(move || {
            let failed = (|| {
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).or_fail()?;
                }
                std::fs::write(path, &data).or_fail()?;
                Ok(())
            })()
            .err();
            let event = Event::Resource(ResourceEvent::Put { id, failed });
            let _ = event_tx.push_custom_event(event);
        });
        id
    }

    fn resource_get(&mut self, name: &ResourceName) -> ActionId {
        let id = self.next_action_id.get_and_increment();
        let path = self.resolve_resource_path(name);
        let event_tx = self.sdl_event.event_sender();
        std::thread::spawn(move || {
            let (data, failed) = match std::fs::read(path) {
                Ok(data) => (Some(data), None),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (None, None),
                Err(e) => (None, Some(Failure::new(e.to_string()))),
            };
            let event = Event::Resource(ResourceEvent::Get { id, data, failed });
            let _ = event_tx.push_custom_event(event);
        });
        id
    }

    fn resource_delete(&mut self, name: &ResourceName) -> ActionId {
        let id = self.next_action_id.get_and_increment();
        let path = self.resolve_resource_path(name);
        let event_tx = self.sdl_event.event_sender();
        std::thread::spawn(move || {
            let failed = std::fs::remove_file(path).err().and_then(|e| {
                (e.kind() != std::io::ErrorKind::NotFound).then(|| Failure::new(e.to_string()))
            });
            let event = Event::Resource(ResourceEvent::Delete { id, failed });
            let _ = event_tx.push_custom_event(event);
        });
        id
    }
}

impl std::fmt::Debug for SdlSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SdlSystem {{ .. }}")
    }
}
