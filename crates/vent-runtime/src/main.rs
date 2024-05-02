use vent_runtime::VentApplication;

fn main() {
    VentApplication::default();
}

#[cfg(target_os = "android")]
use android_activity::{
    input::{InputEvent, KeyAction, KeyEvent, KeyMapChar, MotionAction},
    AndroidApp, InputStatus, MainEvent, PollEvent,
};

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    VentApplication::default();
}
