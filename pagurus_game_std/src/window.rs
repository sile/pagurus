use pagurus::{
    event::{Event, MouseEvent, WindowEvent},
    spatial::{Position, Region, Size},
};

#[derive(Debug)]
pub struct LogicalWindow {
    canvas_region: Region,
    logical_window_size: Size,
    actual_window_size: Size,
}

impl Default for LogicalWindow {
    fn default() -> Self {
        Self::new(Size::default())
    }
}

impl LogicalWindow {
    pub const fn new(canvas_size: Size) -> Self {
        Self {
            canvas_region: Region::new(Position::ORIGIN, canvas_size),
            logical_window_size: canvas_size,
            actual_window_size: Size::square(0),
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Event {
        match event {
            Event::Mouse(event) => Event::Mouse(self.handle_mouse_event(event)),
            Event::Window(event) => Event::Window(self.handle_window_event(event)),
            _ => event,
        }
    }

    pub fn size(&self) -> Size {
        self.logical_window_size
    }

    pub fn canvas_region(&self) -> Region {
        self.canvas_region
    }

    fn handle_mouse_event(&mut self, mut event: MouseEvent) -> MouseEvent {
        let mut position = event.position();

        let logical_window = self.logical_window_size;
        let actual_window = self.actual_window_size;
        let scale_x = logical_window.width as f32 / actual_window.width as f32;
        let scale_y = logical_window.height as f32 / actual_window.height as f32;

        position.x = (position.x as f32 * scale_x).round() as i32;
        position.y = (position.y as f32 * scale_y).round() as i32;

        position.x -= self.canvas_region.position.x as i32;
        position.y -= self.canvas_region.position.y as i32;

        event.set_position(position);
        event
    }

    fn handle_window_event(&mut self, event: WindowEvent) -> WindowEvent {
        if let WindowEvent::RedrawNeeded { size } = event {
            self.actual_window_size = size;

            let canvas = self.canvas_region.size;
            let actual_window = self.actual_window_size;

            self.logical_window_size = canvas;
            if canvas.aspect_ratio() > actual_window.aspect_ratio() {
                let scale = canvas.width as f32 / actual_window.width as f32;
                self.logical_window_size.height =
                    (actual_window.height as f32 * scale).round() as u32;
                let padding = (self.logical_window_size.height - canvas.height) / 2;
                self.canvas_region.position = Position::from_xy(0, padding as i32);
            } else if canvas.aspect_ratio() < actual_window.aspect_ratio() {
                let scale = canvas.height as f32 / actual_window.height as f32;
                self.logical_window_size.width =
                    (actual_window.width as f32 * scale).round() as u32;
                let padding = (self.logical_window_size.width - canvas.width) / 2;
                self.canvas_region.position = Position::from_xy(padding as i32, 0);
            } else {
                self.canvas_region.position = Position::ORIGIN;
            }
        }

        event
    }
}
