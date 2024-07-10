#[cfg(unix)]
#[path = "wayland/mod.rs"]
#[allow(clippy::module_inception)]
pub mod platform;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
#[allow(clippy::module_inception)]
pub mod platform;

pub use self::platform::*;
