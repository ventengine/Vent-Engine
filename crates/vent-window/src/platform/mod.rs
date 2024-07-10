#[cfg(unix)]
#[path = "wayland/mod.rs"]
pub mod platform;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub mod platform;

pub use self::platform::*;
