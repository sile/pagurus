use pagurus::{
    event::{Event, WindowEvent},
    failure::{Failure, OrFail},
    spatial::Size,
    video::VideoFrame,
    Result,
};
use std::{cell::RefCell, sync::mpsc};
use std::{collections::VecDeque, ffi::CString};
use windows::{
    core::PCSTR,
    Win32::UI::WindowsAndMessaging::*,
    Win32::{
        Foundation::{GetLastError, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            GetDC, InvalidateRect, RedrawWindow, ReleaseDC, SetDIBitsToDevice, UpdateWindow,
            ValidateRect, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC, RDW_UPDATENOW,
            RGBQUAD,
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, LoadCursorW,
            PostQuitMessage, RegisterClassA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MSG,
            WINDOW_EX_STYLE, WNDCLASSA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

#[derive(Debug)]
pub struct WindowBuilder {
    title: String,
    window_size: Option<Size>,
}

impl WindowBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            window_size: None,
        }
    }

    pub fn window_size(mut self, size: Option<Size>) -> Self {
        self.window_size = size;
        self
    }

    pub fn build(self) -> Result<Window> {
        let handle = MainWindowThread::spawn(self).or_fail()?;
        Ok(Window::new(handle))
    }
}

#[derive(Debug)]
pub struct Window {
    handle: MainWindowThreadHandle,
    screen_size: Size,
    event_queue: VecDeque<Event>,
    queued_redraw_event_count: usize,
}

impl Window {
    fn new(handle: MainWindowThreadHandle) -> Self {
        Self {
            handle,
            screen_size: Size::EMPTY,
            event_queue: VecDeque::new(),
            queued_redraw_event_count: 0,
        }
    }

    pub fn next_event(&mut self) -> Event {
        for event in self.handle.event_rx.try_iter() {
            if matches!(event, Event::Window(WindowEvent::RedrawNeeded { .. })) {
                self.queued_redraw_event_count += 1;
            }
            self.event_queue.push_back(event);
        }

        while let Some(event) = self.event_queue.pop_front() {
            if let Event::Window(WindowEvent::RedrawNeeded { size }) = &event {
                self.screen_size = *size;
                self.queued_redraw_event_count -= 1;
                if self.queued_redraw_event_count > 0 {
                    continue;
                }
            }
            return event;
        }

        self.handle
            .event_rx
            .recv()
            .unwrap_or_else(|_| unreachable!())
    }

    pub fn draw_video_frame(&mut self, frame: VideoFrame<&[u8]>) -> Result<()> {
        unsafe {
            let hdc = GetDC(self.handle.hwnd);
            (hdc.0 != 0).or_fail()?;
            let mut dc = DeviceContext { window: self, hdc };
            dc.draw_video_frame(frame).or_fail()?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DeviceContext<'a> {
    window: &'a mut Window,
    hdc: HDC,
}

impl<'a> DeviceContext<'a> {
    unsafe fn draw_video_frame(&mut self, frame: VideoFrame<&[u8]>) -> Result<()> {
        let screen_size = get_screen_size(self.window.handle.hwnd).or_fail()?;
        if screen_size != self.window.screen_size {
            return Ok(());
        }

        let frame_size = frame.spec().resolution;
        let stride = frame.spec().stride;
        if screen_size != frame_size {
            // TODO: StretchDIBits
            return Ok(());
        }

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: stride as i32,
            biHeight: -(frame_size.height as i32),
            biPlanes: 1,
            biBitCount: 24,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };
        bmi.bmiColors[0] = RGBQUAD {
            rgbBlue: 255,
            rgbGreen: 255,
            rgbRed: 255,
            rgbReserved: 0,
        };
        SetDIBitsToDevice(
            self.hdc,
            0, // xDest,
            0, // yDest,
            screen_size.width,
            screen_size.height,
            0,                 // xSrc,
            0,                 // ySrc,
            0,                 // StartScan,
            frame_size.height, // cLines,
            frame.data().as_ptr() as _,
            &bmi,
            DIB_RGB_COLORS,
        );

        Ok(())
    }
}

impl<'a> Drop for DeviceContext<'a> {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(self.window.handle.hwnd, self.hdc);
        }
    }
}

#[derive(Debug)]
struct MainWindowThreadHandle {
    hwnd: HWND,
    event_tx: mpsc::Sender<Event>,
    event_rx: mpsc::Receiver<Event>,
}

std::thread_local! {
      static EVENT_TX: RefCell<mpsc::Sender<Event>> = RefCell::new(mpsc::channel().0);
}

#[derive(Debug)]
struct MainWindowThread {
    event_tx: mpsc::Sender<Event>,
}

impl MainWindowThread {
    fn spawn(options: WindowBuilder) -> Result<MainWindowThreadHandle> {
        let (event_tx, event_rx) = mpsc::channel();
        let event_tx_for_handle = event_tx.clone();
        let (hwnd_tx, hwnd_rx) = mpsc::channel();
        std::thread::spawn(move || unsafe {
            EVENT_TX.with(|tx| *tx.borrow_mut() = event_tx.clone());

            let hwnd = match create_window(options).or_fail() {
                Ok(hwnd) => hwnd,
                Err(e) => {
                    let _ = hwnd_tx.send(Err(e));
                    return;
                }
            };
            let _ = hwnd_tx.send(Ok(hwnd));

            let mut message = MSG::default();
            while GetMessageA(&mut message, hwnd, 0, 0).as_bool() {
                DispatchMessageA(&message);
            }
            let _ = event_tx.send(Event::Terminating);
        });

        let hwnd = hwnd_rx.recv().or_fail()?.or_fail()?;
        Ok(MainWindowThreadHandle {
            hwnd,
            event_tx: event_tx_for_handle,
            event_rx,
        })
    }
}

unsafe fn create_window(options: WindowBuilder) -> Result<HWND> {
    let instance = GetModuleHandleA(None).or_fail()?;
    if instance.0 == 0 {
        return Err(Failure::new("Failed to create a module handle".to_owned()));
    }

    let window_class = windows::s!("window");

    let wc = WNDCLASSA {
        hCursor: LoadCursorW(None, IDC_ARROW).or_fail()?,
        hInstance: instance,
        lpszClassName: window_class,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        ..Default::default()
    };
    if RegisterClassA(&wc) == 0 {
        return Err(Failure::new(
            "Failed to register an window class".to_owned(),
        ));
    }

    let (width, height) = if let Some(size) = options.window_size {
        (size.width as i32, size.height as i32)
    } else {
        (CW_USEDEFAULT, CW_USEDEFAULT)
    };

    let hwnd = CreateWindowExA(
        WINDOW_EX_STYLE::default(),
        window_class,
        PCSTR::from_raw(CString::new(options.title).or_fail()?.as_ptr() as _),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        width,
        height,
        None,
        None,
        instance,
        None,
    );

    Ok(hwnd)
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut event = None;
    let mut result = LRESULT(0);
    let mut quit = false;
    match message {
        WM_PAINT => {
            if let Ok(size) = get_screen_size(hwnd) {
                event = Some(Event::Window(WindowEvent::RedrawNeeded { size }));
            } else {
                quit = true;
            }
            ValidateRect(hwnd, None);
        }
        WM_DESTROY => {
            quit = true;
        }
        _ => {
            result = DefWindowProcA(hwnd, message, wparam, lparam);
        }
    }

    if quit {
        event = Some(Event::Terminating);
        PostQuitMessage(0);
    }

    if let Some(event) = event {
        EVENT_TX.with(|tx| {
            if tx.borrow().send(event).is_err() {
                PostQuitMessage(0);
            }
        });
    }
    result
}

unsafe fn get_screen_size(hwnd: HWND) -> Result<Size> {
    let mut rect: RECT = std::mem::zeroed();
    if GetClientRect(hwnd, &mut rect).as_bool() {
        Ok(Size::from_wh(rect.right as u32, rect.bottom as u32))
    } else {
        Err(Failure::new(format!(
            "GetClientRect() error: code={}",
            GetLastError().0
        )))
    }
}
