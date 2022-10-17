use pagurus::{
    failure::{Failure, OrFail},
    spatial::Size,
    Result,
};
use std::ffi::CString;
use std::sync::Mutex;
use windows::{
    core::PCSTR,
    Win32::UI::WindowsAndMessaging::*,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{GetDC, ReleaseDC, ValidateRect, HDC},
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, LoadCursorW,
            PostQuitMessage, RegisterClassA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MSG,
            WINDOW_EX_STYLE, WNDCLASSA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

static WINDOW: Mutex<Option<Window>> = Mutex::new(None);

#[derive(Debug)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn new(title: &str, window_size: Option<Size>) -> Result<Self> {
        let mut global_window = WINDOW.lock().or_fail()?;
        if global_window.is_some() {
            return Err(Failure::new("TODO: message".to_owned()));
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

            *global_window = Some(Self { hwnd }); // TODO
            Ok(Self { hwnd })
        }
    }

    pub fn dispatch(&mut self) -> bool {
        unsafe {
            let mut message = MSG::default();
            if GetMessageA(&mut message, self.hwnd, 0, 0).into() {
                DispatchMessageA(&message);
                true
            } else {
                false
            }
        }
    }

    pub fn get_dc(&self) -> Result<DeviceContext> {
        unsafe {
            let dc = GetDC(self.hwnd);
            (dc.0 != 0).or_fail()?;
            Ok(DeviceContext {
                hwnd: self.hwnd,
                dc,
            })
        }
    }
}

#[derive(Debug)]
pub struct DeviceContext {
    hwnd: HWND,
    dc: HDC,
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe {
            let ret = ReleaseDC(self.hwnd, self.dc);
            assert_eq!(ret, 1);
        }
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let _global_window = WINDOW.lock().map_err(|e| panic!("{e}"));

    unsafe {
        match message {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
