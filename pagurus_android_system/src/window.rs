use ndk::native_window::NativeWindow;
use pagurus::spatial::Size;

pub struct Window<'a> {
    inner: &'a NativeWindow,
}

impl<'a> Window<'a> {
    pub fn new(inner: &'a NativeWindow) -> Self {
        Self { inner }
    }

    pub fn get_window_size(&self) -> Size {
        unsafe {
            let width = ndk_sys::ANativeWindow_getWidth(self.inner.ptr().as_mut());
            assert!(width >= 0, "error-code={width}");

            let height = ndk_sys::ANativeWindow_getHeight(self.inner.ptr().as_mut());
            assert!(height >= 0, "error-code={height}");

            Size {
                width: width as u32,
                height: height as u32,
            }
        }
    }

    pub fn set_buffer_size(&self, size: Size) {
        unsafe {
            let ret = ndk_sys::ANativeWindow_setBuffersGeometry(
                self.inner.ptr().as_mut(),
                size.width as i32,
                size.height as i32,
                ndk_sys::AHardwareBuffer_Format::AHARDWAREBUFFER_FORMAT_R8G8B8_UNORM.0 as i32,
            );
            assert_eq!(ret, 0);
        }
    }

    pub fn acquire_buffer(self) -> Option<WindowBuffer<'a>> {
        unsafe {
            let mut buffer = ndk_sys::ANativeWindow_Buffer {
                width: 0,
                height: 0,
                stride: 0,
                format: 0,
                bits: std::ptr::null_mut(),
                reserved: [0; 6],
            };

            let ret = ndk_sys::ANativeWindow_lock(
                self.inner.ptr().as_mut(),
                (&mut buffer) as *mut _,
                std::ptr::null_mut(),
            );
            if ret == 0 {
                Some(WindowBuffer {
                    window: self,
                    inner: buffer,
                })
            } else {
                println!(
                    "[WARN] [{}:{}] faild to acquire window lock: error_code={}",
                    file!(),
                    line!(),
                    -ret
                );
                None
            }
        }
    }
}

pub struct WindowBuffer<'a> {
    window: Window<'a>,
    inner: ndk_sys::ANativeWindow_Buffer,
}

impl<'a> WindowBuffer<'a> {
    pub fn stride(&self) -> i32 {
        self.inner.stride
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.inner.bits as *mut u8,
                self.inner.height as usize * self.inner.stride as usize * 3,
            )
        }
    }
}

impl<'a> Drop for WindowBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            ndk_sys::ANativeWindow_unlockAndPost(self.window.inner.ptr().as_mut());
        }
    }
}
