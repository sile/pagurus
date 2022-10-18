use pagurus::{
    event::{Event, WindowEvent},
    failure::{Failure, OrFail},
    spatial::Size,
    video::VideoFrame,
    Result,
};
use std::ffi::CString;
use std::sync::mpsc;
use std::sync::Mutex;
use windows::{
    core::PCSTR,
    Win32::UI::WindowsAndMessaging::*,
    Win32::{
        Foundation::{GetLastError, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            GetDC, InvalidateRect, ReleaseDC, SetDIBits, SetDIBitsToDevice, ValidateRect,
            BITMAPINFO, BITMAPINFOHEADER, BI_COMPRESSION, BI_RGB, DIB_RGB_COLORS, HDC, RGBQUAD,
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, LoadCursorW,
            PostQuitMessage, RegisterClassA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MSG,
            WINDOW_EX_STYLE, WNDCLASSA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

static EVENT_TX: Mutex<Option<mpsc::Sender<Event>>> = Mutex::new(None);

#[derive(Debug)]
pub struct Window {
    hwnd: HWND,
    event_rx: mpsc::Receiver<Event>,
}

impl Window {
    pub fn new(title: &str, window_size: Option<Size>) -> Result<Self> {
        {
            let global_window = EVENT_TX.lock().or_fail()?;
            if global_window.is_some() {
                return Err(Failure::new("TODO: message".to_owned()));
            }
        }

        unsafe {
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

            let (width, height) = if let Some(size) = window_size {
                (size.width as i32, size.height as i32)
            } else {
                (CW_USEDEFAULT, CW_USEDEFAULT)
            };
            let hwnd = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                window_class,
                PCSTR::from_raw(CString::new(title.to_owned()).or_fail()?.as_ptr() as _),
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

            let mut global_window = EVENT_TX.lock().or_fail()?;
            let (event_tx, event_rx) = mpsc::channel();

            let _ = event_tx.send(Event::Window(WindowEvent::RedrawNeeded {
                size: get_screen_size(hwnd).or_fail()?,
            }));

            *global_window = Some(event_tx);
            Ok(Self { hwnd, event_rx })
        }
    }

    pub fn next_event(&mut self) -> Event {
        let mut message = MSG::default();
        unsafe {
            loop {
                match self.event_rx.try_recv() {
                    Ok(event) => {
                        return event;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return Event::Terminating;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }

                if GetMessageA(&mut message, self.hwnd, 0, 0).into() {
                    DispatchMessageA(&message);
                } else {
                    return Event::Terminating;
                }
            }
        }
    }

    pub fn get_dc(&self) -> Result<DeviceContext> {
        unsafe {
            let hdc = GetDC(self.hwnd);
            (hdc.0 != 0).or_fail()?;
            Ok(DeviceContext {
                hwnd: self.hwnd,
                hdc,
            })
        }
    }
}

#[derive(Debug)]
pub struct DeviceContext {
    hwnd: HWND,
    hdc: HDC,
}

impl DeviceContext {
    pub fn draw_bitmap(&self, frame: VideoFrame<&[u8]>) -> Result<()> {
        let screen_size = get_screen_size(self.hwnd).or_fail()?;
        let frame_size = frame.spec().resolution;
        if screen_size != frame_size {
            // TODO: StretchDIBits or send redraw-needed event
            return Ok(());
        }

        unsafe {
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: frame_size.width as i32,
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

            InvalidateRect(self.hwnd, None, true).as_bool().or_fail()?;
        }
        Ok(())
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe {
            let ret = ReleaseDC(self.hwnd, self.hdc);
            assert_eq!(ret, 1);
        }
    }
}

fn get_screen_size(hwnd: HWND) -> Result<Size> {
    unsafe {
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
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut event = None;
    let mut result = LRESULT(0);
    let mut quit = false;
    unsafe {
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
            if let Some(tx) = &*EVENT_TX.lock().unwrap_or_else(|e| panic!("{e}")) {
                if tx.send(event).is_err() {
                    PostQuitMessage(0);
                }
            }
        }
    }
    result
}
