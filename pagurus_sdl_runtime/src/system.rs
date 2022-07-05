use pagurus::event::Event;
use pagurus::failure::{Failure, OrFail};
use pagurus::resource::ResourceName;
use pagurus::spatial::Size;
use pagurus::{AudioData, GameRequirements, Result, System, VideoFrame};
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant, UNIX_EPOCH};

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
    sdl_event_pump: EventPump,
    audio_queue: AudioQueue<i16>,
    start: Instant,
    event_queue: VecDeque<Event>,
    timeout_queue: BinaryHeap<(Reverse<Duration>, u64)>,
    options: SdlSystemOptions,
}

impl SdlSystem {
    pub const DEFAULT_TITLE: &'static str = "Pagurus";
    pub const DEFAULT_WINDOW_SIZE: Size = Size::from_wh(800, 600);

    // TODO: with_custom_window
    pub fn new(requirements: GameRequirements, options: SdlSystemOptions) -> Result<Self> {
        let sdl_context = sdl2::init().map_err(Failure::new)?;

        // TODO: SDL_RenderSetLogicalSize, SDL_RenderSetScale
        // Video
        let video_subsystem = sdl_context.video().map_err(Failure::new)?;
        let window_size = requirements
            .window_size
            .unwrap_or(Self::DEFAULT_WINDOW_SIZE);
        let window = video_subsystem
            .window(Self::DEFAULT_TITLE, window_size.width, window_size.height)
            .position_centered()
            .build()
            .or_fail()?;
        let sdl_canvas = window.into_canvas().build().or_fail()?;

        // Audio
        let sdl_audio = sdl_context.audio().map_err(Failure::new)?;
        let audio_spec = AudioSpecDesired {
            freq: Some(AudioData::SAMPLE_RATE as i32),
            channels: Some(AudioData::CHANNELS),
            samples: None,
        };
        let audio_queue = sdl_audio
            .open_queue(None, &audio_spec)
            .map_err(Failure::new)?;
        audio_queue.resume();

        // Event
        let sdl_event_pump = sdl_context.event_pump().map_err(Failure::new)?;

        Ok(Self {
            sdl_canvas,
            sdl_event_pump,
            audio_queue,
            start: Instant::now(),
            event_queue: VecDeque::new(),
            timeout_queue: BinaryHeap::new(),
            options,
        })
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
        self.audio_queue
            .queue_audio(&samples)
            .unwrap_or_else(|e| panic!("failed to queue audio data: {e}"));
        samples.len()
    }

    fn audio_cancel(&mut self) {
        self.audio_queue.clear();
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

    fn clock_set_timeout(&mut self, timeout: Duration, tag: u64) {
        let time = self.start.elapsed() + timeout;
        self.timeout_queue.push((Reverse(time), tag));
    }

    fn resource_put(&mut self, name: &ResourceName, data: &[u8]) {
        self.resolve_resource_path(name);
        //         let action = self.next_action_id;
        //         self.next_action_id = ActionId::new(self.next_action_id.get() + 1);

        //         let event = if uri.starts_with("grn:json:") {
        //             let path = Path::new(RESOURCE_DIR).join(&uri["grn:json:".len()..]);
        //             std::fs::create_dir_all(path.parent().expect("TODO")).unwrap_or_else(|e| {
        //                 // TODO: succeeded=false
        //                 panic!("failed to write to {path:?} file: {e}")
        //             });
        //             std::fs::write(&path, data).unwrap_or_else(|e| {
        //                 // TODO: succeeded=false
        //                 panic!("failed to write to {path:?} file: {e}")
        //             });
        //             Event::Resource(ResourceEvent::Put {
        //                 action,
        //                 succeeded: true,
        //             })
        //         } else {
        //             Event::Resource(ResourceEvent::Put {
        //                 action,
        //                 succeeded: false,
        //             })
        //         };
        //         self.event_queue.push_back(event);
    }

    fn resource_get(&mut self, name: &ResourceName) {
        todo!()
    }

    fn resource_delete(&mut self, name: &ResourceName) {
        todo!()
    }
}

impl std::fmt::Debug for SdlSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SdlSystem {{ .. }}")
    }
}

// use byteorder::{BigEndian, ByteOrder};
// use gazami::{ActionId, Event, GameRequirements, ResourceEvent, System};
// use sdl2::{
//     audio::{AudioQueue, AudioSpecDesired},
//     pixels::PixelFormatEnum,
//     render::Canvas,
//     video::Window,
//     EventPump,
// };
// use std::{
//     collections::VecDeque,
//     num::NonZeroU32,
//     path::Path,
//     time::{Duration, Instant},
// };

// // TODO: option
// const RESOURCE_DIR: &str = "data/";

//     pub fn wait_event(&mut self) -> Event {
//         loop {
//             let timeout = self.last_tick.map_or(0, |last| {
//                 let elapsed = last.elapsed().as_millis() as u32;
//                 let max_wait_ms = 1000 / self.max_fps().get();
//                 max_wait_ms.saturating_sub(elapsed)
//             });
//             if timeout == 0 {
//                 break;
//             }
//             if let Some(event) = self.event_queue.pop_front() {
//                 return event;
//             }

//             if let Some(event) = self.sdl_event_pump.wait_event_timeout(timeout) {
//                 // TODO: consider fixed_aspect_ratio
//                 if let Some(event) = crate::event::sdl_event_to_gazami_event(&event) {
//                     return event;
//                 }
//             } else {
//                 break;
//             }
//         }

//         self.last_tick = Some(Instant::now());
//         Event::Tick
//     }
// }

// impl System for SdlSystem {

//     fn resource_put(&mut self, uri: &str, data: &[u8]) -> ActionId {
//     }

//     fn resource_get(&mut self, uri: &str) -> ActionId {
//         let action = self.next_action_id;
//         self.next_action_id = ActionId::new(self.next_action_id.get() + 1);

//         // TODO: Use I/O thread
//         let event = if uri.starts_with("grn:json:") {
//             let path = Path::new(RESOURCE_DIR).join(&uri["grn:json:".len()..]);
//             if path.exists() {
//                 let data = std::fs::read(&path).unwrap_or_else(|e| {
//                     // TODO: succeeded=false
//                     panic!("failed to read {path:?} file: {e}")
//                 });
//                 Event::Resource(ResourceEvent::Get {
//                     action,
//                     data: Some(data),
//                     succeeded: true,
//                 })
//             } else {
//                 Event::Resource(ResourceEvent::Get {
//                     action,
//                     data: None,
//                     succeeded: true,
//                 })
//             }
//         } else {
//             Event::Resource(ResourceEvent::Get {
//                 action,
//                 data: None,
//                 succeeded: false,
//             })
//         };
//         self.event_queue.push_back(event);

//         action
//     }

//     fn resource_delete(&mut self, _uri: &str) -> ActionId {
//         todo!()
//     }
// }
