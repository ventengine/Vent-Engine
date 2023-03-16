use rfd::MessageLevel;
use std::process::exit;

pub fn crash(desc: String, code: i32) {
    rfd::MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_description(desc.as_str())
        .show();
    exit(code);
}

fn log_crash() {
    todo!()
}
