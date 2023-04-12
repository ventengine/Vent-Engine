use rfd::MessageLevel;
use std::panic::{self, PanicInfo};

#[inline]
pub fn init_panic_hook() {
    panic::set_hook(Box::new(panic_handler));
}

fn panic_handler(pi: &PanicInfo) {
    rfd::MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_description(format!("{pi}").as_str())
        .show();
}

fn log_crash() {
    todo!()
}
