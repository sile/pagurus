#[ndk_glue::main(backtrace = "on")]
pub fn main() {
    #[allow(deprecated)]
    ndk_glue::native_activity().finish()
}
