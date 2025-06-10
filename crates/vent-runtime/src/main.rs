#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // This disables the Windows Console in release mode

use vent_logging::Logger;
use vent_runtime::{VentApplication, util::crash::init_panic_hook};

fn main() {
    init_panic_hook();
    Logger::init();
    let app = VentApplication::default();
    app.start();
}

#[cfg(target_os = "android")]
use android_activity::{
    AndroidApp, InputStatus, MainEvent, PollEvent,
    input::{InputEvent, KeyAction, KeyEvent, KeyMapChar, MotionAction},
};

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    VentApplication::default();
}
