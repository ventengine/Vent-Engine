#[cfg(target_os = "linux")]
#[path = "wayland/mod.rs"]
pub mod platform;

pub use self::platform::*;
