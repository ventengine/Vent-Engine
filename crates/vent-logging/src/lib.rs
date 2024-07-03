use colored::Colorize;
use log::{Level, LevelFilter, Log};

///
/// Cross Platform Logger
///
pub struct Logger {}

impl Logger {
    pub fn new() {
        log::set_max_level(LevelFilter::Debug);
        log::set_boxed_logger(Box::new(Self {})).expect("failed to set boxed logger");
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true // TODO
    }

    fn log(&self, record: &log::Record) {
        // Default
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            let level = match record.level() {
                Level::Error => format!("{:<5}", record.level().to_string().red()),
                Level::Warn => format!("{:<5}", record.level().to_string().yellow()),
                Level::Info => format!("{:<5}", record.level().to_string().cyan()),
                Level::Debug => format!("{:<5}", record.level().to_string().purple()),
                Level::Trace => format!("{:<5}", record.level().to_string().normal()),
            };
            println!("{} {}", level, record.args())
        }
        // Wasm
        #[cfg(target_family = "wasm")]
        {
            match record.level() {
                Level::Error => web_sys::console::error_1(&format!("{}", record.args()).into()),
                Level::Warn => web_sys::console::warn_1(&format!("{}", record.args()).into()),
                Level::Info => web_sys::console::info_1(&format!("{}", record.args()).into()),
                Level::Debug => web_sys::console::debug_1(&format!("{}", record.args()).into()),
                Level::Trace => web_sys::console::trace_1(&format!("{}", record.args()).into()),
            }
        }
        // Android
        #[cfg(target_os = "android")]
        {
            use std::ffi::{c_int, CStr, CString};
            let prio = match record.level() {
                Level::Error => ndk_sys::android_LogPriority::ANDROID_LOG_ERROR,
                Level::Warn => ndk_sys::android_LogPriority::ANDROID_LOG_WARN,
                Level::Info => ndk_sys::android_LogPriority::ANDROID_LOG_INFO,
                Level::Debug => ndk_sys::android_LogPriority::ANDROID_LOG_DEBUG,
                Level::Trace => ndk_sys::android_LogPriority::ANDROID_LOG_VERBOSE,
            };
            unsafe {
                ndk_sys::__android_log_write(
                    prio.0 as c_int,
                    CStr::from("").as_ptr(),
                    record.args().as_ptr(),
                );
            }
        }
    }

    fn flush(&self) {}
}
