use orfail::OrFail;
use std::sync::Mutex;

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        let s = format!($($arg)*);
        unsafe {
            $crate::wasm::consoleLog(s.as_ptr(), s.len() as i32);
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        if let Ok(lock) = $crate::io::PRINTLN_FN.lock() {
            if let Some(f) = &*lock {
                let s = format!($($arg)*);
                f(&s);
            } else {
                std::println!($($arg)*);
            }
        } else {
            std::println!($($arg)*);
        }
    })
}

#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

pub static PRINTLN_FN: Mutex<Option<Box<dyn 'static + Send + Fn(&str)>>> = Mutex::new(None);

pub fn set_println_fn<F>(f: F) -> crate::Result<()>
where
    F: 'static + Send + Fn(&str),
{
    *PRINTLN_FN.lock().or_fail()? = Some(Box::new(f));
    Ok(())
}
